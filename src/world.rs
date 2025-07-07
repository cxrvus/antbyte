use crate::ant::Ant;
use crate::circuit::Circuit;
use crate::matrix::Matrix;

pub struct WorldConfig {
	size_x: usize,
	size_y: usize,
	networks: Vec<Circuit>,
	noise_seed: u32, // todo
}

struct WorldState {
	frame: u32,
	cells: Matrix<bool>, // => color depth = 1
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
