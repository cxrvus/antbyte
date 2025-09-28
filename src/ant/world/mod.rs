pub mod gif_export;
pub mod parser;
pub mod run;

mod ant_tick;

use anyhow::{Result, bail};
use rand::{Rng, SeedableRng, rngs::StdRng};

use std::ops::{Deref, DerefMut};

use super::{Ant, Behavior, BorderMode, StartingPos};

use crate::{
	ant::ColorMode,
	util::{matrix::Matrix, vec2::Vec2u},
};

#[derive(Debug, Clone)]
pub struct WorldConfig {
	pub width: usize,
	pub height: usize,
	pub fps: Option<u32>,
	pub speed: Option<u32>,
	pub ticks: Option<u32>,
	pub looping: bool,
	pub border_mode: BorderMode,
	pub starting_pos: StartingPos,
	pub color_mode: ColorMode,
	pub noise_seed: Option<u32>,
	pub description: String,
}

impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			width: 32,
			height: 32,
			fps: Some(60),
			speed: Some(1),
			ticks: None,
			looping: false,
			border_mode: BorderMode::Wrap,
			starting_pos: StartingPos::Center,
			color_mode: ColorMode::RGBI,
			noise_seed: None,
			description: "".into(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct WorldProperties {
	pub behaviors: [Option<Behavior>; 0x100],
	pub config: WorldConfig,
}

impl Default for WorldProperties {
	fn default() -> Self {
		Self {
			behaviors: [const { None }; 0x100],
			config: Default::default(),
		}
	}
}

pub struct WorldState {
	rng: StdRng,
	tick_count: u32,
	pub cells: Matrix<u8>,
	ants: Vec<Ant>,
	ant_cache: Matrix<bool>,
}

pub struct World {
	properties: WorldProperties,
	pub state: WorldState,
}
impl World {
	pub fn new(properties: WorldProperties) -> Result<Self> {
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
			StartingPos::Center => Vec2u {
				x: width / 2,
				y: height / 2,
			},
			StartingPos::MiddleLeft => Vec2u {
				x: 0,
				y: height / 2,
			},
		};

		if world.properties.behaviors[1].is_some() {
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

	fn get_behavior(&self, id: u8) -> &Option<Behavior> {
		&self.properties.behaviors[id as usize]
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
