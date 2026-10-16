use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Palette(BTreeMap<u8, (u8, u8, u8)>);

impl Palette {
	pub fn parse_color(value: &str) -> Result<(u8, u8, u8)> {
		if value.is_empty() {
			return Ok((0, 0, 0));
		}

		let value = value
			.strip_prefix('#')
			.ok_or(anyhow!("color value must begin with '#'"))?;

		if !value.is_ascii() || !matches!(value.len(), 3 | 6) {
			bail!("color must contain exactly three or six hexadecimal digits");
		}

		let width = value.len() / 3;

		let component = |index| {
			u8::from_str_radix(&value[index * width..(index + 1) * width], 16)
				.map_err(|_| anyhow!("color contains invalid hex digits"))
		};

		let red = component(0)?;
		let green = component(1)?;
		let blue = component(2)?;

		Ok(if width == 1 {
			(red * 0x11, green * 0x11, blue * 0x11)
		} else {
			(red, green, blue)
		})
	}
}

impl Default for Palette {
	fn default() -> Self {
		Self(BTreeMap::from([
			(0x00, (0x00, 0x00, 0x00)), // black
			(0x01, (0xaa, 0x00, 0x00)), // red
			(0x02, (0x00, 0xaa, 0x00)), // green
			(0x03, (0xaa, 0x55, 0x00)), // yellow
			(0x04, (0x00, 0x00, 0xaa)), // blue
			(0x05, (0xaa, 0x00, 0xaa)), // magenta
			(0x06, (0x00, 0xaa, 0xaa)), // cyan
			(0x07, (0xaa, 0xaa, 0xaa)), // white
			(0x08, (0x55, 0x55, 0x55)), // bright black
			(0x09, (0xff, 0x55, 0x55)), // bright red
			(0x0a, (0x55, 0xff, 0x55)), // bright green
			(0x0b, (0xff, 0xff, 0x55)), // bright yellow
			(0x0c, (0x55, 0x55, 0xff)), // bright blue
			(0x0d, (0xff, 0x55, 0xff)), // bright magenta
			(0x0e, (0x55, 0xff, 0xff)), // bright cyan
			(0x0f, (0xff, 0xff, 0xff)), // bright white
		]))
	}
}

impl Deref for Palette {
	type Target = BTreeMap<u8, (u8, u8, u8)>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Palette {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
