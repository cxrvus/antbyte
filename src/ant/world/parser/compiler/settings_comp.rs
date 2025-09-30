use anyhow::{Context, Result, anyhow};

use crate::ant::{
	BorderMode, ColorMode, StartingPos,
	world::{
		WorldConfig,
		parser::{Parser, token::Token},
	},
};

const FPS_CAP: u32 = 60;
const SIZE_CAP: u32 = 0x100;
const SPEED_CAP: u32 = 0x2000;

impl WorldConfig {
	pub fn set_setting(&mut self, key: String, value: Token) -> Result<()> {
		let mut parser = Parser {
			tokens: vec![value],
		};

		parser
			.set_setting(self, &key)
			.with_context(|| format!("for setting '{key}'!"))
	}
}

impl Parser {
	fn set_setting(&mut self, config: &mut WorldConfig, key: &str) -> Result<()> {
		match key {
			key @ ("height" | "width" | "size") => {
				let value = self
					.next_number(Some(SIZE_CAP))?
					.ok_or_else(|| anyhow!("number must be greater than 0"))? as usize;

				match key {
					"width" => config.width = value,
					"height" => config.height = value,
					"size" => {
						config.width = value;
						config.height = value;
					}
					_ => unreachable!(),
				}
			}

			"fps" => config.fps = self.next_number(Some(FPS_CAP))?,
			"speed" => config.speed = self.next_number(Some(SPEED_CAP))?,
			"ticks" => config.ticks = self.next_number(None)?,
			"seed" => config.noise_seed = self.next_number(None)?,

			"loop" => config.looping = self.next_bit()?,

			"border" => config.border_mode = BorderMode::try_from(self.next_ident()?)?,
			"start" => config.starting_pos = StartingPos::try_from(self.next_ident()?)?,
			"colors" => config.color_mode = ColorMode::try_from(self.next_ident()?)?,

			"desc" | "description" => config.description = self.next_str()?,

			other => return Err(anyhow!("unknown setting: '{other}'")),
		}

		Ok(())
	}

	fn next_number(&mut self, max: Option<u32>) -> Result<Option<u32>> {
		let token = self.next_token();

		match token {
			Token::Number(value) => {
				// ensure number is non-zero
				Ok(match value {
					0 => None,
					value => Some(match max {
						None => value,
						Some(max) => value.clamp(1, max),
					}),
				})
			}
			Token::Bit(value) => Ok(match value {
				true => Some(1),
				false => None,
			}),
			token => Err(Self::unexpected(token, "number")),
		}
	}

	#[rustfmt::skip]
	fn next_str(&mut self) -> Result<String> {
		let token = self.next_token();
		if let Token::String(value) = token { Ok(value) }
		else { Err(Self::unexpected(token, "string")) }
	}


	#[rustfmt::skip]
	fn next_bit(&mut self) -> Result<bool> {
		let token = self.next_token();
		if let Token::Bit(value) = token { Ok(value) }
		else { Err(Self::unexpected(token, "bit")) }
	}
}
