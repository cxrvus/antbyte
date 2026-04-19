pub mod pin;
pub mod sub_pin;

use crate::util::{dir::Direction, vec2::Vec2u};

pub mod behavior;

#[derive(Clone, Copy, Default)]
pub enum AntStatus {
	#[default]
	Alive,
	Dead,
}

#[derive(Clone, Copy, Default)]
pub struct Ant {
	pub pos: Vec2u,
	/// principle direction - number between 0 and 7
	pub behavior: u8,
	pub child_behavior: u8,
	pub dir: Direction,
	pub child_dir: Direction,
	pub memory: u8,
	pub child_memory: u8,
	pub status: AntStatus,
	pub birth_tick: u32,
}

impl Ant {
	#[inline]
	pub fn is_alive(&self) -> bool {
		!matches!(self.status, AntStatus::Dead)
	}

	#[inline]
	pub fn die(&mut self) {
		self.status = AntStatus::Dead
	}

	#[inline]
	pub fn age(&self, current_tick: u32) -> u32 {
		current_tick.wrapping_sub(self.birth_tick + 1)
	}
}
