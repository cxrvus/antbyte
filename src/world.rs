use std::rc::Rc;

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
		self.state.frame += 1;

		let mut world_image = self.clone();

		for ant in &self.state.ants {
			ant.tick(&mut world_image);
			todo!();
		}

		todo!()
	}

	pub fn get_archetype(&self, id: usize) -> &Archetype {
		self.config
			.archetypes
			.get(id)
			.expect("invalid archetype ID: {id}")
	}
}
