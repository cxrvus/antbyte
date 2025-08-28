use std::cmp::Ordering;

use anyhow::{Ok, Result, anyhow};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Peripheral {
	Cell,
	CellClear,
	CellNext,

	// universal inputs:
	Time,
	Memory,
	MemoryClear,
	Random,

	Obstacle,

	Kill,

	/// 3 bits indicating number of 45 degrees rotations
	Direction,
	Moving,

	SpawnAnt,

	Die,
}

#[derive(Debug, Default)]
struct PeripheralProperties {
	size: u8,
	io_type: Option<IoType>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum IoType {
	Input,
	Output,
}

const BIT: u8 = 1;
const DIR: u8 = 3;
const NIBBLE: u8 = 4;
const CELL: u8 = NIBBLE;
const ANT_ID: u8 = NIBBLE;
const BYTE: u8 = 8;

impl Peripheral {
	fn properties(&self) -> PeripheralProperties {
		use IoType::*;
		type Props = PeripheralProperties;

		// idea: use Vec / HashMap instead of match and incorporate idents
		let props = match self {
			Peripheral::Cell => Props {
				size: CELL,
				io_type: None,
			},
			Peripheral::CellClear => Props {
				size: BIT,
				io_type: Some(Output),
			},
			Peripheral::CellNext => Props {
				size: CELL,
				io_type: Some(Input),
			},
			Peripheral::Time => Props {
				size: BYTE,
				io_type: Some(Input),
			},
			Peripheral::Memory => Props {
				size: BYTE,
				io_type: None,
			},
			Peripheral::MemoryClear => Props {
				size: BIT,
				io_type: Some(Output),
			},
			Peripheral::Random => Props {
				size: BYTE,
				io_type: Some(Input),
			},
			Peripheral::Obstacle => Props {
				size: BIT,
				io_type: Some(Input),
			},
			Peripheral::Direction => Props {
				size: DIR,
				io_type: None,
			},
			Peripheral::Moving => Props {
				size: BIT,
				io_type: None,
			},
			Peripheral::SpawnAnt => Props {
				size: ANT_ID,
				io_type: Some(Output),
			},
			Peripheral::Kill => Props {
				size: BIT,
				io_type: Some(Output),
			},
			Peripheral::Die => Props {
				size: BIT,
				io_type: Some(Output),
			},
		};

		assert_ne!(props.size, 0);

		props
	}

	fn from_ident(ident: &str) -> Option<Self> {
		match ident {
			"C" | "CELL_" => Some(Self::Cell),
			"CC" | "CLEAR" => Some(Self::CellClear),
			"CN" | "NEXT_CELL_" => Some(Self::CellNext),
			"T" | "CLOCK_" => Some(Self::Time),
			"M" | "MEM_" => Some(Self::Memory),
			"MC" | "CLEAR_MEM" => Some(Self::MemoryClear),
			"R" | "RAND_" => Some(Self::Random),
			"OBS" | "OBSTACLE" => Some(Self::Obstacle),
			"D" | "DIR_" => Some(Self::Direction),
			"MOV" | "MOVING" => Some(Self::Moving),
			"A" | "SPAWN_" => Some(Self::SpawnAnt),
			"KILL" => Some(Self::Kill),
			"DIE" => Some(Self::Die),
			_ => None,
		}
	}

	pub(super) fn is_peripheral_ident(ident: &str) -> bool {
		Self::from_ident(ident).is_some()
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

	const PERIPH_PTN: &str = r"^([A-Z_]{1,10})([0-9a-f])?$";

	pub fn from_ident(ident: String) -> Result<Self> {
		let re = Regex::new(Self::PERIPH_PTN).unwrap();

		// let (periph_string, bit_index) =
		let captures = re
			.captures(&ident)
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
				return Err(anyhow!(
					"may not have a bit index in one-bit peripherals\n(in '{ident}')"
				));
			} else if bit_index >= size {
				return Err(anyhow!(
					"bit index may not exceed peripheral bit capacity:\n{bit_index} >= {size}\n(in '{ident}')"
				));
			}
		};

		Ok(Self {
			peripheral,
			bit: bit_index.unwrap_or_default(),
		})
	}
}
