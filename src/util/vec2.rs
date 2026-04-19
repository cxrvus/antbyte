use std::ops;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec2u {
	pub x: usize,
	pub y: usize,
}

impl Vec2u {
	pub const ZERO: Self = Self { x: 0, y: 0 };

	pub fn sign(self) -> Vec2 {
		let Self { x, y } = self;
		Vec2 {
			x: x as i32,
			y: y as i32,
		}
	}
}

impl ops::Add<Vec2u> for Vec2u {
	type Output = Self;

	fn add(self, other: Vec2u) -> Self::Output {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl ops::Rem<Vec2u> for Vec2u {
	type Output = Self;

	fn rem(self, other: Vec2u) -> Self::Output {
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
	pub fn unsign(self) -> Option<Vec2u> {
		let Self { x, y } = self;
		if x >= 0 && y >= 0 {
			Some(Vec2u {
				x: x as usize,
				y: y as usize,
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
