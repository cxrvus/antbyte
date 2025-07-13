pub mod circuit;
pub mod parser;

use crate::util::vec2::Vec2;
use circuit::Circuit;

#[derive(Default)]
pub struct Ant {
	brain: Circuit,
	is_queen: bool,
	age: u32,
	memory: u32,
	dir: Vec2,
}

impl Ant {
	pub fn new(circuit: Circuit) -> Self {
		Self {
			brain: circuit,
			..Default::default()
		}
	}

	pub fn tick(&self, sensors: Sensors) -> Actions {
		let sensor_bits: u32 = sensors.into();
		let action_bits = self.brain.tick(sensor_bits);
		action_bits.into()
	}
}

pub struct SensorSet {
	clock_mask: u8,
	cell_mask: u8,
	rand_bit_count: u8,
	spawn_bit_count: u8,
}

pub struct Sensors {
	clock: u8,
	next_cell: u8,
	memory: u8,
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
