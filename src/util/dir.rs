use std::ops;

use crate::util::vec2::Vec2;

#[derive(Clone, Copy, Default, Debug)]
pub struct Direction(u8);

impl Direction {
	pub const MAX: u8 = 8;
	pub const INV: u8 = 4;

	#[inline]
	pub fn new(value: u8) -> Self {
		Self(value % Self::MAX)
	}

	#[inline]
	pub fn set(&mut self, value: u8) {
		self.0 = value % Self::MAX;
	}

	#[inline]
	pub fn value(&self) -> u8 {
		self.0
	}

	#[inline]
	pub fn inverted(&self) -> Self {
		Self::new(self.0 + Self::INV)
	}

	pub fn as_vec(&self) -> Vec2 {
		let (x, y) = match self.0 {
			0 => (1, 0),
			1 => (1, 1),
			2 => (0, 1),
			3 => (-1, 1),
			4 => (-1, 0),
			5 => (-1, -1),
			6 => (0, -1),
			7 => (1, -1),
			_ => panic!("dir overflow"),
		};

		Vec2 { x, y }
	}

	#[inline]
	pub fn as_chars(&self) -> (char, char) {
		match self.0 {
			0 => ('>', '>'),
			1 => ('\\', '|'),
			2 => ('\\', '/'),
			3 => ('|', '/'),
			4 => ('<', '<'),
			5 => ('|', '\\'),
			6 => ('/', '\\'),
			7 => ('/', '|'),
			_ => panic!("dir overflow"),
		}
	}
}

impl ops::Add<Direction> for Direction {
	type Output = Self;

	#[inline]
	fn add(self, rhs: Direction) -> Self::Output {
		Self::new(self.0 + rhs.0)
	}
}

impl ops::AddAssign<Direction> for Direction {
	fn add_assign(&mut self, rhs: Direction) {
		*self = *self + rhs
	}
}
