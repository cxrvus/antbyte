pub mod run;

mod ant_tick;
mod config;
mod tick_util;

pub use config::WorldConfig;

use anyhow::{Result, bail};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use std::{
	collections::BTreeMap,
	ops::{Deref, DerefMut},
};

use crate::{
	ant::{Ant, Behavior, BorderMode, StartingPos},
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

pub struct WorldState {
	rng: StdRng,
	tick_count: u32,
	pub cells: Matrix<u8>,
	pub ants: Vec<Ant>,
	ant_cache: Matrix<bool>,
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

		let state = WorldState {
			rng,
			tick_count: 0,
			cells: Matrix::new(width, height),
			ant_cache: Matrix::new(width, height),
			ants: vec![],
		};

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
				x: width / 2,
				y: height / 2,
			},
		};

		if world.properties.behaviors.contains_key(&1) {
			let mut ant = Ant::new(starting_pos, 0, 1);
			ant.grow_up();
			world.spawn(ant);
		} else {
			bail!("no entry point: could not find `ant main` or other ant with ID = 1")
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

		for i in 0..self.ants.len() {
			if self.ants[i].is_alive() {
				self.ant_tick(i);
			}
		}

		self.ants.iter_mut().for_each(|ant| ant.grow_up());
		self.ants.retain(|ant| ant.is_alive());

		let no_ants = self.ants.is_empty();

		let tick_overflow = self
			.config()
			.ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		!(no_ants || tick_overflow)
	}

	pub fn tick_count(&self) -> u32 {
		self.tick_count
	}

	pub fn config(&self) -> &WorldConfig {
		&self.properties.config
	}

	pub fn config_mut(&mut self) -> &mut WorldConfig {
		&mut self.properties.config
	}

	pub fn ants(&self) -> &Vec<Ant> {
		&self.ants
	}

	fn get_behavior(&self, id: u8) -> Option<&Behavior> {
		self.properties.behaviors.get(&id)
	}

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
