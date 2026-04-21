pub mod pin;
pub mod sub_pin;

use crate::util::dir::Direction;

pub mod behavior;

#[derive(Clone, Copy, Default)]
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
		let hashed_tick = hash_u32(current_tick);
		(hashed_tick ^ self.dir.get()) % Direction::MAX
	}
}

fn hash_u32(x: u32) -> u8 {
	let x = x ^ (x >> 16);
	let x = x.wrapping_mul(0x45d9f3b);
	let x = x ^ (x >> 16);
	(x & 0xFF) as u8
}
