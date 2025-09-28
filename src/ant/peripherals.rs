use std::cmp::Ordering;

use anyhow::{Ok, Result, anyhow, bail};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Peripheral {
	// ## cell interaction
	CellClear,
	Cell,
	CellNext,

	// ## universal inputs:
	Time,
	Memory,
	Random,

	// ## ant interaction inputs
	Obstacle,
	Kill,

	// ## ant interaction outputs
	/// 3 bits indicating number of 45 degrees rotations
	Direction,
	Halted,
	SpawnAnt,
	Die,
}

#[derive(Debug, Default)]
pub struct PeripheralProperties {
	pub size: u8,
	pub io_type: Option<IoType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoType {
	Input,
	Output,
}

const BIT: u8 = 1;
const DIR: u8 = 3;
const NIBBLE: u8 = 4;
const CELL: u8 = NIBBLE;
const ANT_ID: u8 = BYTE;
const BYTE: u8 = 8;

impl Peripheral {
	// idea: use Vec / HashMap instead of match and incorporate idents
	pub fn properties(&self) -> PeripheralProperties {
		use IoType::*;
		use Peripheral::*;
		type Props = PeripheralProperties;

		#[rustfmt::skip]
		let props = match self {
			Memory 		=> Props { size: BYTE, io_type: None, },
			Cell 		=> Props { size: CELL, io_type: None, },
			CellNext 	=> Props { size: CELL, io_type: Some(Input), },
			Time 		=> Props { size: BYTE, io_type: Some(Input), },
			Random 		=> Props { size: BYTE, io_type: Some(Input), },
			Obstacle 	=> Props { size: BIT, io_type: Some(Input), },
			CellClear 	=> Props { size: BIT, io_type: Some(Output), },
			Direction 	=> Props { size: DIR, io_type: Some(Output), },
			Halted 		=> Props { size: BIT, io_type: Some(Output), },
			SpawnAnt 	=> Props { size: ANT_ID, io_type: Some(Output), },
			Kill 		=> Props { size: BIT, io_type: Some(Output), },
			Die 		=> Props { size: BIT, io_type: Some(Output), },
		};

		debug_assert_ne!(props.size, 0);

		props
	}

	pub fn from_ident(ident: &str) -> Option<Self> {
		match ident {
			"C" | "CELL_" => Some(Self::Cell),
			"CC" | "CLEAR" => Some(Self::CellClear),
			"CN" | "NEXT_CELL_" => Some(Self::CellNext),
			"CX" | "OBS" | "OBSTACLE" => Some(Self::Obstacle),
			"T" | "CLOCK_" => Some(Self::Time),
			"M" | "MEM_" => Some(Self::Memory),
			"R" | "RAND_" => Some(Self::Random),
			"D" | "DIR_" => Some(Self::Direction),
			"DX" | "HALT" => Some(Self::Halted),
			"A" | "SPAWN_" => Some(Self::SpawnAnt),
			"AK" | "KILL" => Some(Self::Kill),
			"AX" | "DIE" => Some(Self::Die),
			_ => None,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputValue {
	pub output: Peripheral,
	pub value: u8,
}

impl PartialOrd for OutputValue {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for OutputValue {
	fn cmp(&self, other: &Self) -> Ordering {
		self.output
			.cmp(&other.output)
			.then(self.value.cmp(&other.value))
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeripheralBit {
	pub peripheral: Peripheral,
	pub bit: u8,
}

impl PeripheralBit {
	pub fn validate(&self, io_type: &IoType) -> Result<()> {
		let properties = self.peripheral.properties();

		let bit_exceeding_size = self.bit >= properties.size;

		let wrong_io_type = match properties.io_type {
			Some(req_io_type) => req_io_type != *io_type,
			None => false,
		};

		if bit_exceeding_size {
			Err(anyhow!("bit index exceeding size: {self:?}",))
		} else if wrong_io_type {
			Err(anyhow!(
				"wrong Input / Output type for Peripheral: {self:?}",
			))
		} else {
			Ok(())
		}
	}

	const PERIPH_PTN: &str = r"^([A-Z_]+)([0-8])?$";

	pub fn from_ident(ident: &str) -> Result<Self> {
		let re = Regex::new(Self::PERIPH_PTN).unwrap();

		// let (periph_string, bit_index) =
		let captures = re
			.captures(ident)
			.ok_or(anyhow!("'{ident}' is not a valid peripheral"))?;

		let periph_ident = captures.get(1).unwrap().as_str();

		let peripheral = Peripheral::from_ident(periph_ident)
			.ok_or(anyhow!("'{periph_ident}' is not a valid peripheral type"))?;

		let bit_index = captures
			.get(2)
			.map(|m| u8::from_str_radix(m.as_str(), 16).unwrap());

		let size = peripheral.properties().size;

		if let Some(bit_index) = bit_index {
			if size == 1 {
				bail!("may not have a bit index in one-bit peripherals\n(in '{ident}')");
			} else if bit_index >= size {
				bail!(
					"bit index may not exceed peripheral bit capacity:\n{bit_index} >= {size}\n(in '{ident}')"
				);
			}
		};

		Ok(Self {
			peripheral,
			bit: bit_index.unwrap_or_default(),
		})
	}
}
