pub mod circuit;
pub mod parser;

use crate::util::vec2::Vec2;
use circuit::Circuit;

#[derive(Default)]
pub struct AntConfig {
	sensors: SensorConfig,
	config: Circuit,
	is_queen: bool,
}

#[derive(Default)]
pub struct Ant {
	config: AntConfig,
	age: u32,
	memory: u32,
	dir: Vec2,
}

impl Ant {
	pub fn new(config: AntConfig) -> Self {
		Self {
			config,
			..Default::default()
		}
	}

	pub fn tick(&self, sensors: Sensors) -> Actions {
		let sensor_bits: u32 = sensors.into();
		let action_bits = self.config.config.tick(sensor_bits);
		action_bits.into()
	}
}

#[derive(Default)]
pub struct SensorConfig {
	clock_mask: u8,
	cell_mask: u8,
	cell_write_mask: u8,
	rand_bit_count: u8,
	spawn_bit_count: u8,
}

pub struct Sensors {
	clock: u8,
	next_cell: u8,
	memory: u32,
	random: u8,
	ant: bool,
	cell_change: bool,
}

impl From<Sensors> for u32 {
	fn from(value: Sensors) -> Self {
		todo!()
	}
}

pub struct Actions {
	direction: u8,
	cell_value: u8,
	cell_write: bool,
	memory_value: u8,
	memory_write: bool,
	despawn: bool,
	/// Queen Only
	spawn: u8,
}

impl From<u32> for Actions {
	fn from(value: u32) -> Self {
		todo!()
	}
}
