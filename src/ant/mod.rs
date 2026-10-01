pub mod pin;
pub mod sub_pin;

use std::ops::{Deref, DerefMut};

use crate::util::{dir::Direction, hash_u32, rotation::Rotation, vec2::PosOffset};

pub mod behavior;

#[derive(Clone, Copy, Default, Debug)]
pub struct Ant {
	pub behavior: u8,
	pub birth_tick: u32,
	pub birth_rot: Rotation,

	pub counter: u8,
	pub sleep_ticks: u8,
	pub memory: u8,

	pub rotation: Rotation,
	pub pos_offset: PosOffset,

	// todo: exclude from serialization
	pub data: TickData,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TickData {
	pub last_input: u8,
	pub move_dir: Option<Direction>,

	pub tie_breaker: bool,
	pub will_halt: bool,
	pub will_kill: bool,
	pub will_die: bool,
	pub will_sleep: bool,

	pub child_behavior: u8,
	pub child_layer: u8,
	pub child_rotation: Rotation,
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
	pub fn sleeping(&self) -> bool {
		!self.will_sleep && self.sleep_ticks > 0
	}

	#[inline]
	pub fn halted(&self) -> bool {
		self.will_halt || self.sleeping()
	}

	#[inline]
	pub fn look_dir(&self) -> Direction {
		self.rotation.as_dir()
	}

	#[inline]
	pub fn move_dir(&self) -> Option<Direction> {
		self.data.move_dir
	}

	pub fn luck(&self, current_tick: u32, layer: u8) -> u8 {
		let hashed_tick = (hash_u32(current_tick) & 0xFF) as u8;
		let state = (self.look_dir().value() ^ layer) & Direction::MAX;
		let luck = (hashed_tick ^ state) % Direction::MOD;
		let bonus = (self.tie_breaker as u8) << Direction::BITS;
		bonus | luck
	}
}
