use crate::ant::{Ant, circuit::Circuit};
use crate::util::matrix::Matrix;

pub enum BorderMode {
	Collide,
	Despawn,
	// todo: Cycle,
	// todo: Wrap,
}

pub struct WorldConfig {
	width: usize,
	height: usize,
	ant_states: Vec<Circuit>,
	border_mode: BorderMode,
	centered: bool,
	noise_seed: Option<u32>, // todo: add rand crate
}

type Cells = Matrix<u8>;

struct WorldState {
	frame: u32,
	cells: Cells,
	ants: Vec<Ant>,
}

pub struct World {
	config: WorldConfig,
	state: WorldState,
}

impl World {
	pub fn new(config: WorldConfig) -> Self {
		let WorldConfig { width, height, .. } = config;

		let state = WorldState {
			frame: 0,
			cells: Matrix::new(width, height),
			ants: vec![],
		};

		Self { config, state }
	}

	pub fn tick(&mut self) {
		self.state.frame += 1;

		for ant in &self.state.ants {
			let config = ant.config();

			for input in config.inputs {
				todo!()
			}
			let input_bits: u32 = config.inputs.compact();
			todo!();
		}

		todo!()
	}

	pub fn cells(&self) -> &Cells {
		&self.state.cells
	}
}
