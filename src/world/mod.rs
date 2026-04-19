pub mod file_compiler;
pub mod run;

mod tick;
mod tick_ant;
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
	util::{
		dir::Direction,
		grid::Grid,
		vec2::{Coord, Vec2u},
	},
	world::config::ColorMode,
};

#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WorldProperties {
	pub name: Option<String>,
	#[serde(rename = "ants")]
	pub behaviors: BTreeMap<u8, Behavior>,
	#[serde(rename = "cfg")]
	pub config: WorldConfig,
}

pub type Cells = Grid<Cell>;

impl Cells {}

#[derive(Debug, Clone, Default)]
pub struct Cell {
	pub value: u8,
	pub occupied: bool,
	pub expiration: Option<u16>,
}

#[derive(Clone, Default)]
struct AsyncActions {
	kills: Vec<usize>,
	moves: Vec<usize>,
	spawns: Vec<usize>,
	deaths: Vec<usize>,
}

#[derive(Clone)]
pub struct WorldState {
	rng: StdRng,
	tick_count: u32,
	async_actions: AsyncActions,
	pub cells: Cells,
	pub ants: Vec<Ant>,
	pub event_in: u8,
	pub event_out: u8,
	pub ext_input: u8,
	pub ext_output: Vec<u8>,
}

impl WorldState {
	pub fn new(rng: StdRng, width: Coord, height: Coord) -> Self {
		Self {
			rng,
			tick_count: 0,
			async_actions: Default::default(),
			cells: Grid::new(width, height),
			ants: vec![],
			event_in: 0,
			event_out: 0,
			ext_input: 0,
			ext_output: vec![],
		}
	}

	#[inline]
	pub fn tick_count(&self) -> u32 {
		self.tick_count
	}

	#[inline]
	pub fn ants(&self) -> &Vec<Ant> {
		&self.ants
	}

	#[inline]
	fn rng(&mut self) -> u8 {
		self.rng.random()
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
			start_pos,
			start_dir,
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

		let half_width = (width - 1) / 2;
		let half_height = (height - 1) / 2;

		let start_pos = match start_pos {
			StartingPos::TopLeft => Vec2u::ZERO,
			StartingPos::Top => Vec2u {
				x: half_height,
				y: 0,
			},
			StartingPos::Left => Vec2u {
				x: 0,
				y: half_height,
			},
			StartingPos::Center => Vec2u {
				x: half_width,
				y: half_height,
			},
		};

		let ant = if let Some(root_id) = world.properties.behaviors.keys().min() {
			Ant {
				pos: start_pos,
				dir: Direction::new(start_dir),
				behavior: *root_id,
				..Default::default()
			}
		} else {
			bail!("can't run a world with no ants defined")
		};

		world.spawn(ant);

		Ok(world)
	}

	pub fn adjusted_color(&self, color: u8) -> u8 {
		match self.config().color_mode {
			ColorMode::Binary => match color {
				0 => 0x0,
				_ => 0xf,
			},
			ColorMode::RGBI => color,
		}
	}

	#[inline]
	pub fn name(&self) -> Option<String> {
		self.properties.name.clone()
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
	fn get_behavior(&self, id: u8) -> Option<&Behavior> {
		self.properties.behaviors.get(&id)
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
