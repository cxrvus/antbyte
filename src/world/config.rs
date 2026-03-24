use anyhow::{Error, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

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
	pub starting_pos: StartingPos,
	/// max number of ants before additional spawning gets blocked
	pub ant_limit: Option<u32>,
	pub color_mode: ColorMode,
	pub noise_seed: Option<u32>,
	pub hide_title: bool, // TODO: move to renderer
	pub description: String,

	// ## Plugins

	// ### Renderer
	/// rendered frames per second
	pub fps: Option<u32>,
	/// amount of ms to sleep for after end of simulation, i.e. between loops
	pub sleep: Option<u32>,
	/// 16 ASCII characters to render cells, start is value = 0
	/// can  also set to empty string to get a default ASCII palette
	pub ascii: Option<String>,
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
			starting_pos: StartingPos::Center,
			ant_limit: None,
			color_mode: ColorMode::RGBI,
			noise_seed: None,
			hide_title: false,
			description: "".into(),

			fps: Some(FPS_CAP),
			sleep: Some(200),
			ascii: None,
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
