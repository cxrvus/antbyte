use crate::ant::{Ant, circuit::Circuit};
use crate::util::matrix::Matrix;

pub struct WorldConfig {
	size_x: usize,
	size_y: usize,
	circuits: Vec<Circuit>,
	noise_seed: Option<u32>, // todo: add rand crate
}

type Cells = Matrix<u8>;

struct WorldState {
	frame: u32,
	cells: Cells,
	ants: Matrix<Option<Ant>>,
}

pub struct World {
	config: WorldConfig,
	state: WorldState,
}

impl World {
	pub fn new(config: WorldConfig) -> Self {
		let WorldConfig { size_x, size_y, .. } = config;

		let state = WorldState {
			frame: 0,
			cells: Matrix::new(size_x, size_y),
			ants: Matrix::new(size_x, size_y),
		};

		Self { config, state }
	}

	pub fn tick(&mut self) {
		self.state.frame += 1;

		todo!()
	}

	pub fn cells(&self) -> &Cells {
		&self.state.cells
	}
}

pub enum Stimulus {
	Time, // cyclic
	Age,
	Cell,
	NextCell,
	Memory,
	Noise,    // randomness
	Constant, // always true
}

pub enum Reaction {
	Cell,
	MemoryValue,
	MemoryWrite,
	Direction,
	Velocity,
}
