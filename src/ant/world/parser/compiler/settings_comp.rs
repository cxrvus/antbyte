use anyhow::{Result, anyhow};

use crate::ant::{
	BorderMode, ColorMode, StartingPos,
	world::{WorldConfig, parser::token::Token},
};

impl WorldConfig {
	pub(super) fn set_setting(&mut self, key: String, value: Token) -> Result<()> {
		// idea: more elegant match block
		match key.as_str() {
			key @ ("height" | "width" | "size") => {
				if let Token::Number(number) = value {
					match key {
						"width" => self.width = number as usize,
						"height" => self.height = number as usize,
						"size" => {
							self.width = number as usize;
							self.height = number as usize;
						}
						_ => unreachable!(),
					}

					Ok(())
				} else {
					invalid_type(&value, "number (pixel count)", key)
				}
			}
			"border" => {
				if let Token::Ident(border_mode) = value {
					self.border_mode = BorderMode::try_from(border_mode)?;
					Ok(())
				} else {
					invalid_type(&value, "border mode (identifier)", &key)
				}
			}
			"start" => {
				if let Token::Ident(starting_pos) = value {
					self.starting_pos = StartingPos::try_from(starting_pos)?;
					Ok(())
				} else {
					invalid_type(&value, "starting pos (identifier)", &key)
				}
			}
			"colors" => {
				if let Token::Ident(color_mode) = value {
					self.color_mode = ColorMode::try_from(color_mode)?;
					Ok(())
				} else {
					invalid_type(&value, "color mode (identifier)", &key)
				}
			}
			"seed" => {
				if let Token::Number(seed) = value {
					self.noise_seed = Some(seed);
					Ok(())
				} else {
					invalid_type(&value, "seed value (number)", &key)
				}
			}
			"desc" | "description" => {
				if let Token::String(desc) = value {
					self.description = desc;
					Ok(())
				} else {
					invalid_type(&value, "description (string)", &key)
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
