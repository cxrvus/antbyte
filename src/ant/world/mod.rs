pub mod parser;

mod ant_tick;

use rand::{Rng, SeedableRng, rngs::StdRng};

use std::{
	ops::{Deref, DerefMut},
	rc::Rc,
};

use super::{Ant, Behavior, BorderMode, StartingPos};

use crate::util::{matrix::Matrix, vec2::Vec2u};

type Cells = Matrix<u8>;

#[derive(Debug, Clone)]
pub struct WorldConfig {
	pub width: usize,
	pub height: usize,
	pub border_mode: BorderMode,
	pub starting_pos: StartingPos,
	pub noise_seed: Option<u32>,
	pub description: String,
}
impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			width: 32,
			height: 32,
			border_mode: BorderMode::Collide,
			starting_pos: StartingPos::Center,
			noise_seed: None,
			description: "".into(),
		}
	}
}

#[derive(Debug)]
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

#[derive(Clone)]
pub struct WorldState {
	rng: StdRng,
	frame: usize,
	// TODO: store ants in cells
	cells: Cells,
	// TODO: limit ant count
	ants: Vec<Ant>,
}

#[derive(Clone)]
pub struct World {
	properties: Rc<WorldProperties>,
	pub state: WorldState,
}

impl From<WorldProperties> for World {
	fn from(properties: WorldProperties) -> Self {
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

		let mut state = WorldState {
			rng,
			frame: 0,
			cells: Matrix::new(width, height),
			ants: vec![],
		};

		if !properties.behaviors.is_empty() {
			let starting_pos = match starting_pos {
				StartingPos::TopLeft => Vec2u::ZERO,
				StartingPos::Center => Vec2u {
					x: width / 2,
					y: height / 2,
				},
			};

			let mut ant = Ant::new(1, 0);
			ant.pos = starting_pos;
			state.ants.push(ant);
		}

		Self {
			properties: Rc::new(properties),
			state,
		}
	}
}

impl World {
	pub fn tick(&mut self) -> bool {
		// idea: optimize - remove cloning (here and in ant_tick)
		self.frame += 1;

		let mut world_frame = self.clone();

		let live_ants: Vec<&Ant> = self.ants.iter().filter(|ant| ant.alive).collect();

		if live_ants.is_empty() {
			return false;
		}

		for (i, ant) in live_ants.iter().enumerate() {
			let ant = world_frame.ant_tick(ant);
			world_frame.ants[i] = ant;
		}

		*self = world_frame;

		true
	}

	pub fn frame(&self) -> usize {
		self.frame
	}

	pub fn config(&self) -> &WorldConfig {
		&self.properties.config
	}

	pub fn ants(&self) -> &Vec<Ant> {
		&self.ants
	}

	pub fn cells(&self) -> &Matrix<u8> {
		&self.cells
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
