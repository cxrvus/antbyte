use rand::{Rng, SeedableRng, rngs::StdRng};

use std::{
	ops::{Deref, DerefMut},
	rc::Rc,
};

use crate::{
	ant::{Ant, Archetype},
	util::matrix::Matrix,
};

pub enum BorderMode {
	Collide,
	Despawn,
	// todo: Cycle,
	// todo: Wrap,
}

type Cells = Matrix<u8>;

pub struct WorldConfig {
	archetypes: Vec<Archetype>,
	width: usize,
	height: usize,
	border_mode: BorderMode,
	noise_seed: Option<u32>, // todo: add rand crate
	                         // todo: add ant starting position enum
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
			state.ants.push(Ant::new(0));
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
