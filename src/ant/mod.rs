pub mod peripherals;
pub mod world;

pub use crate::parser::compiler;

use crate::{
	ant::peripherals::{IoType, PeripheralBit},
	truth_table::TruthTable,
	util::{
		find_dupe,
		vec2::{Vec2, Vec2u},
	},
};

use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all="snake_case")]
pub enum StartingPos { TopLeft, MiddleLeft, Center }

impl TryFrom<String> for StartingPos {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
		match value.as_str() {
			"top_left" => Ok(Self::TopLeft),
			"mid_left" => Ok(Self::MiddleLeft),
			"center" => Ok(Self::Center),
			invalid => Err(anyhow!("invalid starting pos: '{invalid}'")),
		}
	}
}

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Clone, Copy, Default)]
enum AntStatus {
	#[default]
	Newborn,
	Alive,
	Dead,
}

#[derive(Clone, Copy, Default)]
pub struct Ant {
	pub behavior: u8,
	status: AntStatus,
	pub pos: Vec2u,
	/// principle direction - number between 0 and 7
	pub dir: u8,
	pub halted: bool,
	pub memory: u8,
	pub age: u32,
}

impl Ant {
	pub fn new(pos: Vec2u, dir: u8, behavior: u8) -> Self {
		Self {
			pos,
			dir,
			behavior,
			..Default::default()
		}
	}

	#[inline]
	pub fn is_alive(&self) -> bool {
		!matches!(self.status, AntStatus::Dead)
	}

	pub fn grow_up(&mut self) {
		if matches!(self.status, AntStatus::Newborn) {
			self.status = AntStatus::Alive
		}
	}

	pub fn die(&mut self) {
		self.status = AntStatus::Dead
	}

	pub fn dir_vec(&self) -> Vec2 {
		debug_assert!(self.dir < 8);
		Vec2::PRINCIPAL[self.dir as usize]
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

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(try_from = "BehaviorJSON", into = "BehaviorJSON")]
pub struct Behavior {
	pub name: String,
	pub logic: TruthTable,
	pub inputs: Vec<PeripheralBit>,
	pub outputs: Vec<PeripheralBit>,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
struct BehaviorJSON {
	name: String,
	logic: Vec<u32>,
	inputs: Vec<PeripheralBit>,
	outputs: Vec<PeripheralBit>,
}

impl TryFrom<BehaviorJSON> for Behavior {
	type Error = String;

	fn try_from(value: BehaviorJSON) -> std::result::Result<Self, Self::Error> {
		let logic = TruthTable::new(value.inputs.len(), value.outputs.len(), value.logic)
			.map_err(|e| e.to_string())?;

		Behavior::new(value.name, logic, value.inputs, value.outputs).map_err(|e| e.to_string())
	}
}

impl From<Behavior> for BehaviorJSON {
	fn from(value: Behavior) -> Self {
		Self {
			name: value.name,
			logic: value.logic.entries().clone(),
			inputs: value.inputs,
			outputs: value.outputs,
		}
	}
}

impl Behavior {
	pub fn new(
		name: String,
		truth_table: TruthTable,
		inputs: Vec<PeripheralBit>,
		outputs: Vec<PeripheralBit>,
	) -> Result<Self> {
		if inputs.len() > 8 {
			return Err(anyhow!(
				"may not have more than 8 inputs, got {}\n{:?}:\n",
				inputs.len(),
				inputs
			));
		} else if outputs.len() > 32 {
			return Err(anyhow!(
				"may not have more than 32 inputs, got {}\n{:?}:\n",
				outputs.len(),
				outputs
			));
		}

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
