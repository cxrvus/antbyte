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

#[derive(Debug)]
pub struct World {
	pub behaviors: Vec<Behavior>,
	pub width: usize,
	pub height: usize,
	pub border_mode: BorderMode,
	pub starting_pos: StartingPos,
	pub noise_seed: Option<u32>,
}

impl Default for World {
	fn default() -> Self {
		Self {
			behaviors: vec![],
			width: 32,
			height: 32,
			border_mode: BorderMode::Collide,
			starting_pos: StartingPos::Center,
			noise_seed: None,
		}
	}
}

#[derive(Clone)]
pub struct WorldState {
	rng: StdRng,
	frame: usize,
	// todo: ant cache matrix
	cells: Cells,
	// todo: limit ant count
	ants: Vec<Ant>,
}

#[derive(Clone)]
pub struct WorldInstance {
	config: Rc<World>,
	pub state: WorldState,
}

impl WorldInstance {
	pub fn new(world: World) -> Self {
		let World { width, height, .. } = world;

		let rng = if let Some(seed) = world.noise_seed {
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

		if !world.behaviors.is_empty() {
			let starting_pos = match world.starting_pos {
				StartingPos::TopLeft => Vec2u::ZERO,
				StartingPos::Center => Vec2u {
					x: world.width / 2,
					y: world.height / 2,
				},
			};

			let mut ant = Ant::new(0);
			ant.pos = starting_pos;
			state.ants.push(ant);
		}

		Self {
			config: Rc::new(world),
			state,
		}
	}

	pub fn tick(&mut self) {
		// todo: optimize - remove cloning (here and in ant_tick)
		self.frame += 1;

		let mut world_frame = self.clone();

		for (i, ant) in self.ants.iter().enumerate() {
			let ant = world_frame.ant_tick(ant);
			world_frame.ants[i] = ant;
		}

		*self = world_frame;
	}

	pub fn frame(&self) -> usize {
		self.frame
	}

	pub fn config(&self) -> &World {
		&self.config
	}

	pub fn ants(&self) -> &Vec<Ant> {
		&self.ants
	}

	pub fn cells(&self) -> &Matrix<u8> {
		&self.cells
	}

	fn get_behavior(&self, id: u32) -> Option<&Behavior> {
		self.config.behaviors.get(id as usize)
	}

	fn rng(&mut self) -> u8 {
		self.rng.random()
	}
}

impl Deref for WorldInstance {
	type Target = WorldState;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

impl DerefMut for WorldInstance {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.state
	}
}
