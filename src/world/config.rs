use anyhow::{Error, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::ant::MAX_DIR;

pub const FPS_CAP: u32 = 50;
pub const SIZE_CAP: u32 = 0x400;
pub const SPEED_CAP: u32 = 0x2000;

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WorldConfig {
	// ## Core
	/// width in pixels
	pub width: usize,
	/// height in pixels
	pub height: usize,
	/// simulated ticks per frame (defaults to 1)
	pub speed: Option<u32>,
	/// simulation tick limit
	pub ticks: Option<u32>,
	/// amount of ticks after which a cell will automatically reset
	pub decay: Option<u16>,
	/// re-run simulation after it ends
	pub looping: bool,
	/// behavior if ants touch the worlds border
	pub border_mode: BorderMode,
	/// position of the first ant
	pub start_pos: StartingPos,
	pub start_dir: u8,
	/// max number of ants before additional spawning gets blocked
	pub ant_limit: Option<u32>,
	pub color_mode: ColorMode,
	pub noise_seed: Option<u32>,
	pub description: String,

	// ## Plugins

	// ### Renderer
	/// rendered frames per second
	pub fps: Option<u32>,
	/// don't render title banner
	pub hide_title: bool, // TODO: move to renderer
	/// don't render ants
	pub hide_ants: bool,
	/// amount of ms to sleep for after end of simulation, i.e. between loops
	pub sleep: Option<u32>,
	/// 16 ASCII characters to render cells, start is value = 0
	/// can  also set to empty string to get a default ASCII palette
	pub ascii: Option<String>,

	// ### External Input
	/// 1 to 8 characters as key bindings, representing K0-K7 in ascending order
	pub keys: Option<String>,
}

impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			width: 32,
			height: 32,
			speed: Some(1),
			ticks: None,
			decay: None,
			looping: false,
			border_mode: BorderMode::Wrap,
			start_pos: StartingPos::Center,
			start_dir: 0,
			ant_limit: None,
			color_mode: ColorMode::RGBI,
			noise_seed: None,
			description: "".into(),

			fps: Some(FPS_CAP),
			hide_title: false,
			hide_ants: false,
			sleep: Some(200),
			ascii: None,

			keys: None,
		}
	}
}

#[rustfmt::skip]
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderMode { Collide, Despawn, Cycle, Wrap }

impl TryFrom<String> for BorderMode {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"obs" | "collide" => Ok(Self::Collide),
			"die" | "despawn" => Ok(Self::Despawn),
			"cycle" => Ok(Self::Cycle),
			"wrap" => Ok(Self::Wrap),
			invalid => Err(anyhow!("invalid border mode: '{invalid}'")),
		}
	}
}

#[rustfmt::skip]
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum StartingPos { TopLeft, Top, Left, Center }

impl TryFrom<String> for StartingPos {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"top_left" => Ok(Self::TopLeft),
			"top" => Ok(Self::Top),
			"left" => Ok(Self::Left),
			"center" => Ok(Self::Center),
			invalid => Err(anyhow!("invalid starting pos: '{invalid}'")),
		}
	}
}

#[rustfmt::skip]
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode { Binary, RGBI }

impl TryFrom<String> for ColorMode {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"rgb" | "rbgi" => Ok(Self::RGBI),
			"bin" => Ok(Self::Binary),
			invalid => Err(anyhow!("invalid starting pos: '{invalid}'")),
		}
	}
}

impl WorldConfig {
	pub fn validate(&self) -> Result<()> {
		Self::non_zero(self.height, "height")?;
		Self::non_zero(self.width, "width")?;
		Self::cap(self.height, "height", SIZE_CAP)?;
		Self::cap(self.width, "width", SIZE_CAP)?;

		if self.start_dir > MAX_DIR {
			bail!("starting direction must not exceed {MAX_DIR}")
		}

		Self::cap(
			self.start_dir as usize,
			"start direction",
			(MAX_DIR - 1) as u32,
		)?;

		Self::cap_opt(self.fps, "FPS", FPS_CAP)?;
		Self::cap_opt(self.speed, "speed", SPEED_CAP)?;
		Self::cap_opt(self.sleep, "sleep", 10000)?;

		if let Some(ascii) = &self.ascii {
			let ascii_len = ascii.len();
			if !ascii.is_empty() && ascii_len != 16 {
				bail!(
					"the ascii setting must be None, an empty string, or 16 characters long. found {ascii_len}"
				);
			};
		};

		if let Some(keys) = &self.keys
			&& keys.len() > 8
		{
			bail!("can only specify up to 8 keys. found {}", keys.len())
		}

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
