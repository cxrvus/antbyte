use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::ant::{BorderMode, ColorMode, StartingPos};

pub const FPS_CAP: u32 = 50;
pub const SIZE_CAP: u32 = 0x400;
pub const SPEED_CAP: u32 = 0x2000;

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WorldConfig {
	/// width in pixels
	pub width: usize,
	/// height in pixels
	pub height: usize,
	/// rendered frames per second
	pub fps: Option<u32>,
	/// simulated ticks per frame (defaults to 1)
	pub speed: Option<u32>,
	/// simulation tick limit
	pub ticks: Option<u32>,
	/// amount of ms to sleep for after end of simulation, i.e. between loops
	pub sleep: Option<u32>,
	/// re-run simulation after it ends
	pub looping: bool,
	/// behavior if ants touch the worlds border
	pub border_mode: BorderMode,
	/// position of the first ant
	pub starting_pos: StartingPos,
	/// max number of ants before additional spawning gets blocked
	pub ant_limit: Option<u32>,
	pub color_mode: ColorMode,
	pub noise_seed: Option<u32>,
	pub hide_title: bool, // TODO: move to renderer
	pub description: String,
}

impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			width: 32,
			height: 32,
			fps: Some(FPS_CAP),
			speed: Some(1),
			ticks: None,
			sleep: Some(200),
			looping: false,
			border_mode: BorderMode::Wrap,
			starting_pos: StartingPos::Center,
			ant_limit: None,
			color_mode: ColorMode::RGBI,
			noise_seed: None,
			hide_title: false,
			description: "".into(),
		}
	}
}

impl WorldConfig {
	pub fn validate(&self) -> Result<()> {
		Self::non_zero(self.height, "height")?;
		Self::non_zero(self.width, "width")?;
		Self::cap(self.height, "height", SIZE_CAP)?;
		Self::cap(self.width, "width", SIZE_CAP)?;

		Self::cap_opt(self.fps, "FPS", FPS_CAP)?;
		Self::cap_opt(self.speed, "speed", SPEED_CAP)?;
		Self::cap_opt(self.sleep, "sleep", 10000)?;

		Ok(())
	}

	#[inline]
	#[rustfmt::skip]
	fn cap(number: usize, property: &str, max: u32) -> Result<()> {
		if number > max as usize { bail!("[{property}] must not exceed {max}") } Ok(())
	}

	fn cap_opt(number: Option<u32>, property: &str, max: u32) -> Result<()> {
		Self::cap(number.unwrap_or_default() as usize, property, max)
	}

	#[inline]
	#[rustfmt::skip]
	fn non_zero(number: usize, property: &str) -> Result<()> {
		if number == 0 { bail!("[{property}] must be greater than 0"); } Ok(())
	}
}
