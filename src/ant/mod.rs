pub mod peripherals;
pub mod world;

pub use world::parser::compiler;

use crate::{
	ant::peripherals::{IoType, PeripheralBit},
	truth_table::TruthTable,
	util::{
		find_dupe,
		vec2::{Vec2, Vec2u},
	},
};

use anyhow::{Error, Result, anyhow};

#[rustfmt::skip]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum StartingPos { TopLeft, Center }

impl TryFrom<String> for StartingPos {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"top_left" => Ok(Self::TopLeft),
			"center" => Ok(Self::Center),
			invalid => Err(anyhow!("invalid starting pos: '{invalid}'")),
		}
	}
}

#[derive(Debug, Clone)]
pub enum ColorMode {
	Binary,
	RGBI,
}

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

#[derive(Clone, Copy, Default)]
pub enum AntStatus {
	#[default]
	Newborn,
	Alive,
	Dead,
}

#[derive(Clone, Copy, Default)]
pub struct Ant {
	pub behavior: u8,
	pub status: AntStatus,
	pub pos: Vec2u,
	/// principle direction - number between 0 and 7
	pub dir: u8,
	pub halted: bool,
	pub memory: u8,
	pub age: u32,
}

impl Ant {
	pub fn new(behavior: u8, dir: u8) -> Self {
		Self {
			behavior,
			dir,
			..Default::default()
		}
	}

	pub fn die(&mut self) {
		self.status = AntStatus::Dead;
	}

	pub fn get_dir_vec(&self) -> Vec2 {
		debug_assert!(self.dir < 8);
		Vec2::principal()[self.dir as usize]
	}

	#[inline]
	pub fn set_dir(&mut self, dir: u8) {
		self.dir = Self::wrap_dir(dir);
	}

	#[inline]
	pub fn flip_dir(&mut self) {
		self.set_dir(self.dir + 4);
	}

	#[inline]
	fn wrap_dir(dir: u8) -> u8 {
		dir % 8
	}
}

#[derive(Debug, Clone)]
pub struct Behavior {
	pub name: String,
	pub logic: TruthTable,
	pub inputs: Vec<PeripheralBit>,
	pub outputs: Vec<PeripheralBit>,
}

impl Behavior {
	pub fn new(
		name: String,
		truth_table: TruthTable,
		inputs: Vec<PeripheralBit>,
		outputs: Vec<PeripheralBit>,
	) -> Result<Self> {
		Self::validate_periphs(&inputs, IoType::Input)?;
		Self::validate_periphs(&outputs, IoType::Output)?;

		Ok(Self {
			logic: truth_table,
			name,
			inputs,
			outputs,
		})
	}

	pub fn validate_periphs(periphs: &Vec<PeripheralBit>, io_type: IoType) -> Result<()> {
		if let Some(dupe) = find_dupe(periphs) {
			Err(anyhow!("found duplicate peripheral in Behavior: {dupe:?}"))
		} else {
			for periph in periphs {
				periph.validate(&io_type)?;
			}

			Ok(())
		}
	}
}
