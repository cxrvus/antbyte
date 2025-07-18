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
	centered: bool,
	noise_seed: Option<u32>, // todo: add rand crate
}

#[derive(Clone)]
pub struct WorldState {
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

		let state = WorldState {
			frame: 0,
			cells: Matrix::new(width, height),
			ants: vec![],
		};

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

		todo!()
	}

	pub fn border_mode(&self) -> &BorderMode {
		&self.config.border_mode
	}

	pub fn get_archetype(&self, id: usize) -> Option<&Archetype> {
		self.config.archetypes.get(id)
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
