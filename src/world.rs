use rand::{Rng, SeedableRng, rngs::StdRng};

use std::{
	ops::{Deref, DerefMut},
	rc::Rc,
};

use crate::{
	ant::{Ant, archetype::Archetype},
	util::{matrix::Matrix, vec2::Vec2u},
};

pub enum BorderMode {
	Collide,
	Despawn,
	// todo: Cycle,
	// todo: Wrap,
}

pub enum StartingPos {
	TopLeft,
	Center,
}

type Cells = Matrix<u8>;

pub struct WorldConfig {
	pub archetypes: Vec<Archetype>,
	width: usize,
	height: usize,
	border_mode: BorderMode,
	starting_pos: StartingPos,
	noise_seed: Option<u32>, // todo: add rand crate
}

impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			archetypes: vec![],
			width: 16,
			height: 16,
			border_mode: BorderMode::Collide,
			starting_pos: StartingPos::Center,
			noise_seed: None,
		}
	}
}

#[derive(Clone)]
pub struct WorldState {
	rng: StdRng,
	frame: u32,
	pub cells: Cells,
	pub ants: Vec<Ant>,
}

#[derive(Clone)]
pub struct World {
	config: Rc<WorldConfig>,
	pub state: WorldState,
}

impl World {
	pub fn new(config: WorldConfig) -> Self {
		let WorldConfig { width, height, .. } = config;

		let rng = if let Some(seed) = config.noise_seed {
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

		if !config.archetypes.is_empty() {
			let starting_pos = match config.starting_pos {
				StartingPos::TopLeft => Vec2u::ZERO,
				StartingPos::Center => Vec2u {
					x: config.width / 2,
					y: config.height / 2,
				},
			};

			let mut ant = Ant::new(0);
			ant.pos = starting_pos;
			state.ants.push(ant);
		}

		Self {
			config: Rc::new(config),
			state,
		}
	}

	pub fn tick(&mut self) {
		self.frame += 1;

		let mut world_image = self.clone();

		for ant in self.ants.iter_mut().filter(|ant| ant.is_alive()) {
			ant.tick(&mut world_image);
			todo!();
		}

		*self = world_image;
	}

	pub fn border_mode(&self) -> &BorderMode {
		&self.config.border_mode
	}

	pub fn get_archetype(&self, id: u32) -> Option<&Archetype> {
		self.config.archetypes.get(id as usize)
	}

	pub fn rng(&mut self) -> u32 {
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
