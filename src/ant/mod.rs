pub mod pin;
pub mod sub_pin;

use std::ops::{Deref, DerefMut};

use crate::util::{dir::Direction, hash_u32};

pub mod behavior;

#[derive(Clone, Copy, Default, Debug)]
pub struct Ant {
	pub behavior: u8,
	pub birth_tick: u32,

	pub clock: u8,
	pub wait_ticks: u8,
	pub dir: Direction,
	pub memory: u8,

	// todo: exclude from serialization
	pub data: TickData,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TickData {
	pub last_input: u8,

	pub will_halt: bool,
	pub will_dash: bool,
	pub will_kill: bool,
	pub will_die: bool,
	pub will_wait: bool,

	pub child_behavior: u8,
	pub child_layer: u8,
	pub child_dir: Direction,
	pub child_memory: u8,
}

impl Deref for Ant {
	type Target = TickData;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl DerefMut for Ant {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
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

	pub fn luck(&self, current_tick: u32, layer: u8) -> u8 {
		let hashed_tick = (hash_u32(current_tick) & 0xFF) as u8;
		let state = (self.dir.value() ^ layer) & Direction::MAX;
		let luck = (hashed_tick ^ state) % Direction::MOD;
		let bonus = (self.will_dash as u8) << Direction::BITS;
		bonus | luck
	}
}
