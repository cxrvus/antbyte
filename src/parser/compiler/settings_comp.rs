use anyhow::{Context, Result, anyhow};

use crate::{
	parser::{Parser, token::Token},
	world::config::{BorderMode, ColorMode, StartingPos, WorldConfig},
};

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
				let value = self.next_number()?.ok_or(anyhow!(
					"size settings must be greater than zero.\nfound in: {key}"
				))? as usize;

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

			"fps" => config.fps = self.next_number()?,
			"speed" => config.speed = self.next_number()?,
			"decay" => config.decay = self.next_number().map(|x| x.map(|v| v as u16))?,
			"sleep" => config.sleep = self.next_number()?,
			"ticks" => config.ticks = self.next_number()?,
			"noise_seed" | "seed" => config.noise_seed = self.next_number()?,

			"dur" => {
				// set tick limit: ticks = duration (seconds) * speed (ticks / frame) * fps (frames / second)
				if let Some(fps) = config.fps
					&& let Some(speed) = config.speed
				{
					let duration = self
						.next_number()?
						.ok_or(anyhow!("duration must be greater than 0"))?;

					let ticks = duration.saturating_mul(speed).saturating_mul(fps);
					config.ticks = Some(ticks);
				}
			}

			"looping" | "loop" => config.looping = self.next_bit()?,

			"border_mode" | "border" => {
				config.border_mode = BorderMode::try_from(self.next_ident()?)?
			}

			"starting_pos" | "start" => {
				config.starting_pos = StartingPos::try_from(self.next_ident()?)?
			}

			"ant_limit" => config.ant_limit = self.next_number()?,

			"color_mode" | "colors" => config.color_mode = ColorMode::try_from(self.next_ident()?)?,

			"desc" | "description" => config.description = self.next_str()?,

			"ascii" => config.ascii = Some(self.next_str()?),

			other => return Err(anyhow!("unknown setting: '{other}'")),
		}

		// double-check config validity
		config.validate()?;

		Ok(())
	}

	fn next_number(&mut self) -> Result<Option<u32>> {
		let token = self.next_token();

		match token {
			Token::Number(value) => {
				// ensure number is non-zero
				Ok(if value == 0 { None } else { Some(value) })
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
