use crate::ant::{Ant, circuit::Circuit};
use crate::util::matrix::Matrix;

pub enum OutputMode {
	Hex,
	// todo: Ascii(u8),
	// todo: VGA(u8),
}

pub enum BorderMode {
	Collide,
	Despawn,
	// todo: Cycle,
	// todo: Wrap,
}

pub struct WorldConfig {
	size_x: usize,
	size_y: usize,
	circuits: Vec<Circuit>,
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
		let WorldConfig { size_x, size_y, .. } = config;

		let state = WorldState {
			frame: 0,
			cells: Matrix::new(size_x, size_y),
			ants: vec![],
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
