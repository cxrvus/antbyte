use anyhow::{Context, Result, anyhow};

use crate::{
	parser::{Parser, token::Token},
	util::vec2::Coord,
	world::config::{BorderMode, ByteFilter, RenderMask, StartingPos, WorldConfig},
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
					"width" => config.width = value as Coord,
					"height" => config.height = value as Coord,
					"size" => {
						config.width = value as Coord;
						config.height = value as Coord;
					}
					_ => unreachable!(),
				}
			}

			"fps" => config.fps = self.next_number()?,
			"speed" => config.speed = self.next_number()?,
			"decay" => config.decay = self.next_number().map(|x| x.map(|v| v as u16))?,
			"sleep" => config.sleep = self.next_number()?,
			"ticks" => config.max_ticks = self.next_number()?,
			"seed" => config.seed = self.next_number()?,

			"dur" => {
				// set tick limit: ticks = duration (seconds) * speed (ticks / frame) * fps (frames / second)
				if let Some(fps) = config.fps
					&& let Some(speed) = config.speed
				{
					let duration = self
						.next_number()?
						.ok_or(anyhow!("duration must be greater than 0"))?;

					let ticks = duration.saturating_mul(speed).saturating_mul(fps);
					config.max_ticks = Some(ticks);
				}
			}

			"looping" | "loop" => config.looping = self.next_bit()?,

			"border" => config.border = BorderMode::try_from(self.next_ident()?)?,

			"start_pos" | "start" => config.start_pos = StartingPos::try_from(self.next_ident()?)?,
			"start_dir" => config.start_dir = self.next_number()?.unwrap_or_default() as u8,
			"start_tick" => config.start_tick = self.next_number()?.unwrap_or_default(),
			"ant_limit" => config.ant_limit = self.next_number()?,

			"bg_filter" => config.bg_filter = ByteFilter::try_from(self.next_ident()?)?,
			"bg" => config.bg = RenderMask::try_from(self.next_ident()?)?,
			"fg" => config.fg = RenderMask::try_from(self.next_ident()?)?,

			"desc" | "description" => config.description = self.next_str()?,
			"keys" => config.keys = Some(self.next_str()?),

			other => return Err(anyhow!("unknown setting: '{other}'")),
		}

		// double-check config validity
		config.validate()?;

		Ok(())
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
