pub mod pin;
pub mod sub_pin;

use crate::util::vec2::{Vec2, Vec2u};

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
	pub dir: u8,
	pub child_dir: u8,
	pub memory: u8,
	pub child_memory: u8,
	pub status: AntStatus,
	pub birth_tick: u32,
}

pub const MAX_DIR: u8 = 8;

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
	pub fn dir_vec(&self) -> Vec2 {
		debug_assert!(self.dir < 8);
		Vec2::PRINCIPAL[self.dir as usize]
	}

	#[inline]
	pub fn age(&self, current_tick: u32) -> u32 {
		current_tick.wrapping_sub(self.birth_tick + 1)
	}

	#[inline]
	pub fn set_dir(&mut self, dir: u8) {
		self.dir = Self::wrap_dir(dir);
	}

	#[inline]
	pub fn flip_dir(&mut self) {
		self.set_dir(self.dir + 4);
	}

	#[inline]
	pub fn wrap_dir(dir: u8) -> u8 {
		dir % MAX_DIR
	}
}
