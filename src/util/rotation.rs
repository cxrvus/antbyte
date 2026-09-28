use crate::util::dir::Direction;
use anyhow::{Result, bail};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rotation(u8);

impl Rotation {
	pub fn from(value: u8) -> Self {
		Self(value)
	}

	#[inline]
	pub fn value(&self) -> u8 {
		self.0
	}

	#[inline]
	pub fn reset(&mut self) {
		self.0 = 0;
	}

	pub fn rotate(&mut self, value: u8, left: bool) {
		if value > 0 {
			self.0 = if left {
				self.0.wrapping_sub(value)
			} else {
				self.0.wrapping_add(value)
			}
		}
	}

	pub fn as_dir(&self) -> Direction {
		Direction::from_u8(match self.0.wrapping_add(16) {
			0x00..=0x1f => 0,
			0x20..=0x3f => 1,
			0x40..=0x5f => 2,
			0x60..=0x7f => 3,
			0x80..=0x9f => 4,
			0xa0..=0xbf => 5,
			0xc0..=0xdf => 6,
			0xe0..=0xff => 7,
		})
	}

	#[inline]
	pub fn try_from_str(value: &str) -> Result<Self> {
		Ok(Self(
			(match value {
				"E" => 0,
				"SE" => 1,
				"S" => 2,
				"SW" => 3,
				"W" => 4,
				"NW" => 5,
				"N" => 6,
				"NE" => 7,
				_ => bail!(
					"invalid direction string: '{value}'. Use a compass direction, like N, NE, W, etc."
				),
			}) * 32,
		))
	}
}
