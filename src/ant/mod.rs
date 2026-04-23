pub mod pin;
pub mod sub_pin;

use crate::util::{dir::Direction, hash_u32};

pub mod behavior;

#[derive(Clone, Copy, Default, Debug)]
pub struct Ant {
	pub birth_tick: u32,

	pub behavior: u8,
	pub child_behavior: u8,
	pub dir: Direction,
	pub child_dir: Direction,
	pub memory: u8,
	pub child_memory: u8,

	pub halt: bool,
	pub die: bool,
	pub kill: bool,
}

impl Ant {
	#[inline]
	pub fn age(&self, current_tick: u32) -> u32 {
		current_tick.wrapping_sub(self.birth_tick + 1)
	}

	#[inline]
	pub fn luck(&self, current_tick: u32) -> u8 {
		let hashed_tick = (hash_u32(current_tick) & 0xFF) as u8;
		(hashed_tick ^ self.dir.get()) % Direction::MAX
	}
}
