use std::collections::BTreeMap;

use anyhow::{Error, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::util::{dir::Direction, vec2::Coord};

pub const FPS_CAP: u32 = 50;
pub const SPEED_CAP: u32 = 0x4000;
pub const SIZE_CAP: Coord = 0x200;
pub const LAYER_CAP: u8 = 8;
const ANT_LIMIT: u32 = 0x400;

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WorldConfig {
	// ## Core
	/// width in pixels
	pub width: Coord,
	/// height in pixels
	pub height: Coord,
	/// number of ant layers
	pub layers: u8,
	/// layer that will be rendered
	pub main_layer: u8,
	/// simulated ticks per frame (defaults to 1)
	pub speed: Option<u32>,
	/// simulation tick limit
	pub max_ticks: Option<u32>,
	/// amount of ticks after which a cell will automatically reset
	pub decay: Option<u16>,
	/// re-run simulation after it ends
	pub looping: bool,
	/// behavior if ants touch the worlds border
	pub border: BTreeMap<u8, BorderMode>,
	/// position of the first ant
	pub start_pos: StartingPos,
	/// first tick to render
	pub start_tick: u32,
	/// direction value (0-7) for start ant
	pub start_dir: u8,
	/// max number of ants before additional spawning gets blocked
	pub ant_limit: u32,
	pub seed: Option<u32>,
	pub description: String,

	// ## Plugins

	// ### Renderer
	/// rendered frames per second
	pub fps: Option<u32>,
	/// filter to get desired nibble (4 bits) out of BG byte
	pub bg_filter: ByteFilter,
	/// background render mask
	pub bg: RenderMask,
	/// foreground render mask
	pub fg: RenderMask,
	/// amount of ms to sleep for after end of simulation, i.e. between loops
	pub sleep: Option<u32>,

	// ### External Input
	/// 1 to 8 characters as key bindings, representing K0-K7 in ascending order
	pub keys: Option<String>,
}

impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			width: 16,
			height: 16,
			layers: 1,
			main_layer: 0,
			speed: Some(1),
			max_ticks: None,
			decay: None,
			looping: false,
			border: BTreeMap::from([(0, BorderMode::Wrap)]),
			start_pos: StartingPos::Center,
			start_dir: 0,
			ant_limit: ANT_LIMIT,
			seed: None,
			description: "".into(),

			fps: Some(FPS_CAP),
			start_tick: 0,
			bg_filter: ByteFilter::Lsb,
			bg: RenderMask::Cell,
			fg: RenderMask::Dir,
			sleep: Some(200),

			keys: None,
		}
	}
}

#[rustfmt::skip]
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[serde(rename_all = "snake_case")]
pub enum StartingPos { TopLeft, Top, TopRight, Left, Center, Right, BottomLeft, Bottom, BottomRight  }

impl TryFrom<String> for StartingPos {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"top_left" => Ok(Self::TopLeft),
			"top" => Ok(Self::Top),
			"top_right" => Ok(Self::TopRight),
			"left" => Ok(Self::Left),
			"center" => Ok(Self::Center),
			"right" => Ok(Self::Right),
			"bottom_left" => Ok(Self::BottomLeft),
			"bottom" => Ok(Self::Bottom),
			"bottom_right" => Ok(Self::BottomRight),

			invalid => Err(anyhow!("invalid starting pos: '{invalid}'")),
		}
	}
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
/// filter specified nibble from a byte
pub enum ByteFilter {
	/// least significant
	Lsb,
	/// most significant
	Msb,
	/// 0 if input is 0, else 15
	Bin,
}

impl TryFrom<String> for ByteFilter {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"lsb" => Ok(Self::Lsb),
			"msb" => Ok(Self::Msb),
			"bin" => Ok(Self::Bin),

			invalid => Err(anyhow!("invalid render mask: '{invalid}'")),
		}
	}
}

impl ByteFilter {
	pub fn apply(&self, input: u8) -> u8 {
		match self {
			ByteFilter::Lsb => input & 0xf,
			ByteFilter::Msb => (input & 0xf0) >> 4,
			ByteFilter::Bin => match input {
				0 => 0,
				_ => 0xf,
			},
		}
	}
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RenderMask {
	None,
	Cell,
	Layers,

	// ## Ant
	Dir,
	Id,
	BirthTick,
	InputBits,
	Mem,
}

impl TryFrom<String> for RenderMask {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"none" => Ok(Self::None),
			"cell" => Ok(Self::Cell),
			"layers" => Ok(Self::Layers),
			"dir" => Ok(Self::Dir),
			"id" => Ok(Self::Id),
			"birth_tick" => Ok(Self::BirthTick),
			"input_bits" => Ok(Self::InputBits),
			"mem" => Ok(Self::Mem),

			invalid => Err(anyhow!("invalid render mask: '{invalid}'")),
		}
	}
}

impl WorldConfig {
	pub fn validate(&self) -> Result<()> {
		if self.height < 3 || self.width < 3 {
			bail!("height / width must not be less than 3")
		}

		Self::cap(self.height as u32, "height", SIZE_CAP as u32)?;
		Self::cap(self.width as u32, "width", SIZE_CAP as u32)?;

		Self::cap(self.layers as u32, "layers", LAYER_CAP as u32)?;

		if self.layers == 0 {
			bail!("specified layer count must be greater than 0")
		} else if self.main_layer >= self.layers {
			bail!("main_layer must not exceed specified max layer")
		}

		Self::cap(self.ant_limit, "ant_limit", ANT_LIMIT)?;

		if self.ant_limit < self.layers.into() {
			bail!("ant_limit must not be less than specified layers")
		}

		Self::cap(self.start_dir as u32, "start_dir", Direction::MAX as u32)?;

		if let Some(max_ticks) = self.max_ticks
			&& self.start_tick > max_ticks
		{
			bail!(
				"start tick ({}) must not exceed set tick limit ({max_ticks})",
				self.start_tick
			)
		}

		if self.speed.is_none() {
			bail!("speed must be greater than 0")
		}

		Self::cap_opt(self.fps, "FPS", FPS_CAP)?;
		Self::cap_opt(self.speed, "speed", SPEED_CAP)?;
		Self::cap_opt(self.sleep, "sleep", 10000)?;

		// either FG or BG must be something

		if let RenderMask::None = self.bg
			&& let RenderMask::None = self.fg
		{
			bail!("need to render either fg or bg or both. found both set to [none]")
		}

		if let Some(keys) = &self.keys
			&& keys.len() > 8
		{
			bail!("can only specify up to 8 keys. found {}", keys.len())
		}

		Ok(())
	}

	#[inline]
	#[rustfmt::skip]
	fn cap(number: u32, property: &str, max: u32) -> Result<()> {
		if number > max { bail!("[{property}] must not exceed {max}") } Ok(())
	}

	fn cap_opt(number: Option<u32>, property: &str, max: u32) -> Result<()> {
		Self::cap(number.unwrap_or_default(), property, max)
	}
}
