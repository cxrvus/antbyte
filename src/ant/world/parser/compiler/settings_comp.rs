use anyhow::{Result, anyhow};

use crate::ant::world::{World, parser::token::Token};

impl World {
	pub(super) fn set_setting(&mut self, key: String, value: Token) -> Result<()> {
		let key = key.to_ascii_lowercase();

		// todo: implement all WorldConfig properties
		// idea: more elegant match block
		match key.as_str() {
			key @ "width" | key @ "height" => {
				if let Token::Number(number) = value {
					*match key {
						"width" => &mut self.width,
						"height" => &mut self.height,
						_ => unreachable!(),
					} = number as usize;
					Ok(())
				} else {
					invalid_type(&value, "number (pixel count)", key)
				}
			}
			other => Err(anyhow!("unknown setting: {}", other)),
		}
	}
}

pub fn invalid_type(actual: &Token, expected: &str, key: &str) -> Result<()> {
	Err(anyhow!(
		"expected {expected}, got {actual:?}\nfor key {key}"
	))
}
