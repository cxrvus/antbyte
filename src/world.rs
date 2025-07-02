use crate::ant::{Ant, Circuit};
use crate::map::Map;

pub struct WorldConfig {
	size_x: u8,
	size_y: u8,
	networks: Vec<Circuit>,
	noise_seed: u32, // todo
}

struct WorldState {
	frame: u32,
	cells: Map<bool>, // => color depth = 1
	ants: Map<Option<Ant>>,
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
			cells: Map::new(size_x as usize, size_y as usize),
			ants: Map::new(size_x as usize, size_y as usize),
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
