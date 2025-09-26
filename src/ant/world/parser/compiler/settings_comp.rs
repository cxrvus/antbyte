use anyhow::{Result, anyhow, bail};

use crate::ant::{
	BorderMode, ColorMode, StartingPos,
	world::{WorldConfig, parser::token::Token},
};

impl WorldConfig {
	pub(super) fn set_setting(&mut self, key: String, value: Token) -> Result<()> {
		// todo: more elegant match block
		match key.as_str() {
			key @ ("height" | "width" | "size") => {
				if let Token::Number(value) = value {
					expect_non_zero(value, key)?;

					let value = value.clamp(1, 0x100) as usize;

					match key {
						"width" => self.width = value,
						"height" => self.height = value,
						"size" => {
							self.width = value;
							self.height = value;
						}
						_ => unreachable!(),
					}

					Ok(())
				} else {
					invalid_type(&value, "number (pixel count)", key)
				}
			}
			"fps" => {
				if let Token::Number(value) = value {
					self.fps = non_zero(value, Some(60));

					Ok(())
				} else {
					invalid_type(&value, "number (FPS)", &key)
				}
			}
			"tpf" => {
				if let Token::Number(value) = value {
					self.tpf = non_zero(value, Some(0x100));

					Ok(())
				} else {
					invalid_type(&value, "number (TPF / ticks per frame)", &key)
				}
			}
			"ticks" => {
				if let Token::Number(value) = value {
					self.ticks = non_zero(value, None);

					Ok(())
				} else {
					invalid_type(&value, "number (max ticks)", &key)
				}
			}
			"loop" => {
				match value {
					Token::Bit(value) => self.looping = value,
					value => return invalid_type(&value, "bit (is looping?)", &key),
				}
				Ok(())
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
			other => Err(anyhow!("unknown setting: '{other}'")),
		}
	}
}

#[inline]
fn non_zero(value: u32, max: Option<u32>) -> Option<u32> {
	match value {
		0 => None,
		value => Some(match max {
			None => value,
			Some(max) => value.clamp(1, max),
		}),
	}
}

#[inline]
fn expect_non_zero(value: u32, key: &str) -> Result<()> {
	match value > 0 {
		true => Ok(()),
		false => Err(anyhow!("number must be greater than 0 for setting '{key}'")),
	}
}

#[inline]
fn invalid_type(actual: &Token, expected: &str, key: &str) -> Result<()> {
	bail!("expected {expected}, got {actual:?} for setting '{key}'");
}
