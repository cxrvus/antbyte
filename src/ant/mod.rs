pub mod circuit;
pub mod parser;

use crate::util::{bitvec::BitVec, vec2::Vec2};
use circuit::Circuit;

#[derive(Default)]
pub struct Ant {
	brain: Circuit,
	is_queen: bool,
	age: u32,
	memory: BitVec,
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
		let sensor_bits: BitVec = sensors.into();
		let action_bits = self.brain.tick(&sensor_bits);
		action_bits.into()
	}
}

pub struct Sensors {
	time: u8,
	age: u8,
	current_cell: u8,
	next_cell: u8,
	memory: u8,
	random: u8,
	ant: bool,
}

impl From<Sensors> for BitVec {
	fn from(value: Sensors) -> Self {
		todo!()
	}
}

pub struct Actions {
	direction: u8,
	cell_value: u8,
	cell_write: u8,
	memory_value: u8,
	memory_write: u8,
	despawn: bool,
	/// Queen Only
	spawn: u8,
}

impl From<BitVec> for Actions {
	fn from(value: BitVec) -> Self {
		todo!()
	}
}
