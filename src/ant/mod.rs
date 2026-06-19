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
	pub wait_ticks: u8,

	pub dir: Direction,

	pub will_halt: bool,
	pub will_dash: bool,
	pub will_kill: bool,
	pub will_die: bool,
	pub will_wait: bool,

	pub memory: u8,

	pub child_behavior: u8,
	pub child_layer: u8,
	pub child_dir: Direction,
	pub child_memory: u8,
}

impl Ant {
	#[inline]
	pub fn waiting(&self) -> bool {
		!self.will_wait && self.wait_ticks > 0
	}

	#[inline]
	pub fn halted(&self) -> bool {
		self.will_halt || self.waiting()
	}

	pub fn luck(&self, current_tick: u32) -> u8 {
		let hashed_tick = (hash_u32(current_tick) & 0xFF) as u8;
		let luck = (hashed_tick ^ self.dir.value()) % Direction::MOD;
		let bonus = (self.will_dash as u8) << Direction::BITS;
		bonus | luck
	}
}
