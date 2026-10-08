use anyhow::{Context, Result, anyhow, bail};

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
		let key_parts = key.rsplit_once('_');

		let (key, sub_index) = match key_parts {
			Some((key, suffix)) => match u8::from_str_radix(suffix, 8) {
				Ok(index) => (key.to_owned(), index),
				Err(_) => (format!("{key}_{suffix}"), 0),
			},
			None => (key.to_owned(), 0),
		};

		match key.as_str() {
			key @ ("height" | "width" | "size") => {
				let value = self.next_opt_number()?.ok_or(anyhow!(
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

			"layer_limit" => config.layer_limit = self.next_number()? as u8,
			"layer_filter" => config.layer_filter = self.next_number()? as u8,

			"stepped" => config.stepped = self.next_bit()?,
			"fps" => config.fps = self.next_number()?,
			"speed" => config.speed = self.next_number()?,
			"decay" => config.decay = self.next_opt_number().map(|x| x.map(|v| v as u16))?,
			"sleep" => config.sleep = self.next_opt_number()?,
			"ticks" => config.max_ticks = self.next_opt_number()?,
			"seed" => config.seed = self.next_opt_number()?,

			"dur" => {
				// set tick limit: ticks = duration (seconds) * speed (ticks / frame) * fps (frames / second)
				let duration = self
					.next_opt_number()?
					.ok_or(anyhow!("duration must be greater than 0"))?;

				let ticks = duration
					.saturating_mul(config.speed)
					.saturating_mul(config.fps);
				config.max_ticks = Some(ticks);
			}

			"looping" | "loop" => config.looping = self.next_bit()?,

			"inv_rot" => config.inv_rot = self.next_bit()?,
			"euclid" => config.euclid = self.next_bit()?,

			"border" => {
				let border_mode = BorderMode::try_from(self.next_ident()?)?;
				config.border.insert(sub_index, border_mode);
			}

			"slow_down" | "sldn" => {
				let sldn = self.next_number()? as u16;
				config.slow_down.insert(sub_index, sldn);
			}

			"start_pos" | "start" => config.start_pos = StartingPos::try_from(self.next_ident()?)?,
			"start_dir" => {
				let dir = self.next_number()?;

				if dir > 7 {
					bail!("start_dir must be a number between 0 and 7");
				} else {
					config.start_dir = (dir as u8) * 32;
				}
			}

			"start_tick" => config.start_tick = self.next_number()?,
			"ant_limit" => config.ant_limit = self.next_number()?,

			"bg_filter" => config.bg_filter = ByteFilter::try_from(self.next_ident()?)?,
			"bg" => config.bg = RenderMask::try_from(self.next_ident()?)?,
			"fg" => config.fg = RenderMask::try_from(self.next_ident()?)?,

			"desc" | "description" => config.description = self.next_str()?,

			"keys" => {
				let keys = self.next_str()?;
				config.keys = if keys.is_empty() { None } else { Some(keys) };
			}

			#[rustfmt::skip]
			"midi_out_ch" => {
				_ = config.midi.out_ch.insert(sub_index, self.next_number()? as u8)
			},

			#[rustfmt::skip]
			"midi_out_offset" => {
				_ = config.midi.offset.insert(sub_index, self.next_number()? as u8)
			},

			// "midi_out_offset" => config.midi.offset = self.next_number()?.unwrap_or_default() as u8,
			other => bail!(anyhow!("unknown setting: '{other}'")),
		}

		// double-check config validity
		config.validate()?;

		Ok(())
	}
}
