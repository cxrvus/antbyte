pub mod pin;

use crate::util::vec2::{Vec2, Vec2u};

pub mod behavior;

#[derive(Clone, Copy, Default)]
pub enum AntStatus {
	#[default]
	Newborn,
	Alive,
	Dead,
}

#[derive(Clone, Copy, Default)]
pub struct Ant {
	/// principle direction - number between 0 and 7
	pub dir: u8,
	pub behavior: u8,
	pub status: AntStatus,
	pub pos: Vec2u,
	pub memory: u8,
	pub age: u32,
	pub is_queen: bool,
}

impl Ant {
	pub fn new(pos: Vec2u, dir: u8, behavior: u8, memory: u8) -> Self {
		Self {
			pos,
			behavior,
			memory,
			dir: Self::wrap_dir(dir),
			..Default::default()
		}
	}

	pub fn new_queen(pos: Vec2u) -> Self {
		Self {
			pos,
			is_queen: true,
			status: AntStatus::Alive,
			..Default::default()
		}
	}

	#[inline]
	pub fn is_alive(&self) -> bool {
		!matches!(self.status, AntStatus::Dead)
	}

	pub fn grow_up(&mut self) {
		if matches!(self.status, AntStatus::Newborn) {
			self.status = AntStatus::Alive
		}
	}

	pub fn die(&mut self) {
		self.status = AntStatus::Dead
	}

	pub fn dir_vec(&self) -> Vec2 {
		debug_assert!(self.dir < 8);
		Vec2::PRINCIPAL[self.dir as usize]
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
	fn wrap_dir(dir: u8) -> u8 {
		dir % 8
	}
}
