use std::{thread, time::Duration};

pub fn sleep(secs: f32) {
	thread::sleep(Duration::from_secs_f32(secs));
}

use std::ops;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2 {
	pub x: i32,
	pub y: i32,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec2u {
	pub x: usize,
	pub y: usize,
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

	pub fn cardinal() -> [Vec2; 4] {
		[-Vec2::Y, Vec2::X, Vec2::Y, -Vec2::X]
	}

	pub fn as_str(&self) -> &str {
		let Self { x, y } = self;
		match (x, y) {
			(0, 0) => "o",
			(0, -1) => "^",
			(1, 0) => ">",
			(0, 1) => "v",
			(-1, 0) => "<",
			_ => "*",
		}
	}

	pub const X: Self = Self { x: 1, y: 0 };
	pub const Y: Self = Self { x: 0, y: 1 };
	pub const ZERO: Self = Self { x: 0, y: 0 };
}

impl Vec2u {
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

impl ops::Mul<usize> for Vec2u {
	type Output = Self;

	fn mul(self, scalar: usize) -> Self::Output {
		Self {
			x: self.x * scalar,
			y: self.y * scalar,
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

impl ops::Sub<Vec2> for Vec2 {
	type Output = Self;

	fn sub(self, other: Vec2) -> Self::Output {
		Self {
			x: self.x - other.x,
			y: self.y - other.y,
		}
	}
}

impl ops::Mul<i32> for Vec2 {
	type Output = Self;

	fn mul(self, scalar: i32) -> Self::Output {
		Self {
			x: self.x * scalar,
			y: self.y * scalar,
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

impl ops::Neg for Vec2 {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
		}
	}
}
