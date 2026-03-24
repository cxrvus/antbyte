pub mod file_compiler;
pub mod run;

mod ant_tick;
mod tick_util;

pub mod config;
use config::{StartingPos, WorldConfig};

use anyhow::{Result, bail};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use std::{
	collections::BTreeMap,
	ops::{Deref, DerefMut},
};

use crate::{
	ant::{Ant, behavior::Behavior},
	util::{matrix::Matrix, vec2::Vec2u},
};

#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct WorldProperties {
	pub name: Option<String>,
	#[serde(rename = "ants")]
	pub behaviors: BTreeMap<u8, Behavior>,
	#[serde(rename = "cfg")]
	pub config: WorldConfig,
}

pub type Cells = Matrix<Cell>;

impl Cells {}

#[derive(Debug, Clone, Default)]
pub struct Cell {
	pub value: u8,
	pub occupied: bool,
	pub expiration: Option<u16>,
}

pub struct WorldState {
	rng: StdRng,
	tick_count: u32,
	pub cells: Cells,
	pub queen: Option<Ant>,
	pub ants: Vec<Ant>,
	pub event_in: u8,
	pub event_out: u8,
	pub ext_input: u8,
	pub ext_output: Vec<u8>,
}

impl WorldState {
	pub fn new(rng: StdRng, width: usize, height: usize) -> Self {
		Self {
			rng,
			tick_count: 0,
			cells: Matrix::new(width, height),
			queen: None,
			ants: vec![],
			event_in: 0,
			event_out: 0,
			ext_input: 0,
			ext_output: vec![],
		}
	}
}

pub struct World {
	properties: WorldProperties,
	pub state: WorldState,
}

impl World {
	pub fn new(properties: WorldProperties) -> Result<Self> {
		properties.config.validate()?;

		let WorldConfig {
			width,
			height,
			starting_pos,
			noise_seed,
			..
		} = properties.config.clone();

		let rng = if let Some(seed) = noise_seed {
			StdRng::seed_from_u64(seed as u64)
		} else {
			StdRng::from_seed(rand::random::<[u8; 32]>())
		};

		let state = WorldState::new(rng, width, height);

		let mut world = Self { properties, state };

		let starting_pos = match starting_pos {
			StartingPos::TopLeft => Vec2u::ZERO,
			StartingPos::Top => Vec2u {
				x: height / 2,
				y: 0,
			},
			StartingPos::Left => Vec2u {
				x: 0,
				y: height / 2,
			},
			StartingPos::Center => Vec2u {
				x: width / 2 - 1,
				y: height / 2 - 1,
			},
		};

		if world.properties.behaviors.contains_key(&0) {
			// validate pins for queen

			let behavior = &world.properties.behaviors[&0];
			let queen_pins = [behavior.inputs.clone(), behavior.outputs.clone()].concat();

			if let Some(forbidden) = queen_pins.iter().find(|x| !x.pin.definition().queen) {
				bail!("forbidden pin for queen ant: {:?}", forbidden.pin);
			}

			world.queen = Some(Ant::new_queen(starting_pos));
		} else if world.properties.behaviors.contains_key(&1) {
			let mut ant = Ant::new(starting_pos, 0, 1, 0);
			ant.grow_up();
			world.spawn(ant);
		} else {
			bail!("no entry point: could not find `ant main` or other ant with ID = 0 or 1")
		}

		Ok(world)
	}

	pub fn frame_tick(&mut self) -> bool {
		for _ in 0..self
			.config()
			.speed
			.expect("speed must be greater than 0 to use frame_tick")
		{
			if !self.tick() {
				return false;
			}
		}

		true
	}

	pub fn tick(&mut self) -> bool {
		self.tick_count += 1;

		self.event_in = self.event_out;
		self.event_out = 0;

		if self.queen.is_some() {
			self.ant_tick(None);
		}

		for i in 0..self.ants.len() {
			if self.ants[i].is_alive() {
				self.ant_tick(Some(i));
			}
		}

		// todo: optimize defragmentation
		self.ants.iter_mut().for_each(|ant| ant.grow_up());
		self.ants.retain(|ant| ant.is_alive());

		if let Some(queen) = self.queen
			&& !queen.is_alive()
		{
			self.queen = None;
		}

		// todo: optimize decay
		if self.config().decay.is_some() {
			self.cell_decay();
		}

		let no_ants = self.ants.is_empty() && self.queen.is_none();

		let tick_overflow = self
			.config()
			.ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		!(no_ants || tick_overflow)
	}

	#[rustfmt::skip]
	pub fn set_value(&mut self, pos: &Vec2u, value: u8) {
		let old_cell = self.cells.at(&pos.sign()).unwrap();

		let expiration = match self.config().decay {
			Some(decay) if value != 0 => {
				let clock = self.tick_count as u16;
				Some(clock.wrapping_add(decay))
			}
			_ => None
		};

		let cell = Cell { value, expiration, ..*old_cell };

		self.cells.set_at(&pos.sign(), cell);
	}

	#[rustfmt::skip]
	#[inline]
	pub(super) fn occupy(&mut self, pos: &Vec2u, occupied: bool) {
		let old_cell = self.cells.at(&pos.sign()).unwrap();
		let cell = Cell { occupied, ..*old_cell };
		self.cells.set_at(&pos.sign(), cell);
	}

	fn cell_decay(&mut self) {
		let clock = self.tick_count as u16;

		self.cells
			.entries
			.iter_mut()
			.filter(|cell| cell.expiration == Some(clock))
			.for_each(|cell| {
				cell.value = 0;
				cell.expiration = None;
			});
	}

	#[inline]
	pub fn name(&self) -> Option<String> {
		self.properties.name.clone()
	}

	#[inline]
	pub fn tick_count(&self) -> u32 {
		self.tick_count
	}

	#[inline]
	pub fn config(&self) -> &WorldConfig {
		&self.properties.config
	}

	#[inline]
	pub fn config_mut(&mut self) -> &mut WorldConfig {
		&mut self.properties.config
	}

	#[inline]
	pub fn ants(&self) -> &Vec<Ant> {
		&self.ants
	}

	#[inline]
	fn get_behavior(&self, id: u8) -> Option<&Behavior> {
		self.properties.behaviors.get(&id)
	}

	#[inline]
	fn rng(&mut self) -> u8 {
		self.rng.random()
	}
}

impl Deref for World {
	type Target = WorldState;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

impl DerefMut for World {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.state
	}
}
