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
}
