pub mod pin;
pub mod sub_pin;

use crate::util::{dir::Direction, hash_u32};

pub mod behavior;

#[derive(Clone, Copy, Default, Debug)]
pub struct Ant {
	pub behavior: u8,
	pub birth_tick: u32,

	pub last_input: u8,
	pub clock: u8,

	pub dir: Direction,
	pub halt: bool,
	pub dash: bool,

	pub die: bool,
	pub kill: bool,

	pub memory: u8,

	pub child_behavior: u8,
	pub child_dir: Direction,
	pub child_memory: u8,
}

impl Ant {
	#[inline]
	pub fn luck(&self, current_tick: u32) -> u8 {
		let hashed_tick = (hash_u32(current_tick) & 0xFF) as u8;
		let luck = (hashed_tick ^ self.dir.value()) % Direction::MOD;
		let bonus = (self.dash as u8) << Direction::BITS;
		bonus | luck
	}
}
