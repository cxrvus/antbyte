use std::ops;

use crate::util::hash_u32;

pub type Coord = u16;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
	pub x: Coord,
	pub y: Coord,
}

impl Position {
	pub const ZERO: Self = Self { x: 0, y: 0 };

	pub fn sign(self) -> Vec2 {
		let Self { x, y } = self;
		Vec2 {
			x: x as i32,
			y: y as i32,
		}
	}

	pub fn hash(self) -> u32 {
		// idea: better interlacing
		hash_u32((self.y as u32) << 16 & (self.x as u32))
	}
}

impl ops::Add<Position> for Position {
	type Output = Self;

	fn add(self, other: Position) -> Self::Output {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl ops::Rem<Position> for Position {
	type Output = Self;

	fn rem(self, other: Position) -> Self::Output {
		Self {
			x: self.x.rem_euclid(other.x),
			y: self.y.rem_euclid(other.y),
		}
	}
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2 {
	pub x: i32,
	pub y: i32,
}

impl Vec2 {
	pub fn unsign(self) -> Option<Position> {
		let Self { x, y } = self;
		if x >= 0 && y >= 0 {
			Some(Position {
				x: x as Coord,
				y: y as Coord,
			})
		} else {
			None
		}
	}
}

impl ops::Add<Vec2> for Vec2 {
	type Output = Self;

	fn add(self, other: Vec2) -> Self::Output {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl ops::Rem<Vec2> for Vec2 {
	type Output = Self;

	fn rem(self, other: Vec2) -> Self::Output {
		Self {
			x: self.x.rem_euclid(other.x),
			y: self.y.rem_euclid(other.y),
		}
	}
}
