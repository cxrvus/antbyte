use super::AntType;

use anyhow::{Ok, Result, anyhow};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Peripheral {
	/// Worker Only
	Cell,
	/// Worker Only
	CellClear,
	CellNext,

	Time,
	Memory,
	MemoryClear,
	Random,
	Obstacle,

	/// 3 bits indicating number of 45 degrees rotations
	Direction,
	Moving,

	/// Queen Only
	SpawnAnt,
	/// Queen Only
	Kill,

	Die,
}

#[derive(Debug, Default)]
struct PeripheralProperties {
	size: u8,
	io_type: Option<IoType>,
	forbidden: Vec<AntType>,
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
		type Props = PeripheralProperties;
		use AntType::*;
		use IoType::*;

		let props = match self {
			Peripheral::Cell => Props {
				size: CELL,
				forbidden: vec![Queen],
				..Default::default()
			},
			Peripheral::CellClear => Props {
				size: BIT,
				io_type: Some(Output),
				forbidden: vec![Queen],
			},
			Peripheral::CellNext => Props {
				size: CELL,
				io_type: Some(Input),
				..Default::default()
			},
			Peripheral::Time => Props {
				size: BYTE,
				io_type: Some(Input),
				..Default::default()
			},
			Peripheral::Memory => Props {
				size: BYTE,
				..Default::default()
			},
			Peripheral::MemoryClear => Props {
				size: BIT,
				io_type: Some(Output),
				..Default::default()
			},
			Peripheral::Random => Props {
				size: BYTE,
				io_type: Some(Input),
				..Default::default()
			},
			Peripheral::Obstacle => Props {
				size: BIT,
				io_type: Some(Input),
				..Default::default()
			},
			Peripheral::Direction => Props {
				size: DIR,
				..Default::default()
			},
			Peripheral::Moving => Props {
				size: BIT,
				..Default::default()
			},
			Peripheral::SpawnAnt => Props {
				size: ANT_ID,
				io_type: Some(Output),
				forbidden: vec![Worker],
			},
			Peripheral::Kill => Props {
				size: BIT,
				io_type: Some(Output),
				forbidden: vec![Worker],
			},
			Peripheral::Die => Props {
				size: BIT,
				io_type: Some(Output),
				..Default::default()
			},
		};

		assert_ne!(props.size, 0);

		props
	}

	fn from_ident(ident: &str) -> Option<Self> {
		match ident {
			"c" | "cell_" => Some(Self::Cell),
			"cc" | "clear" => Some(Self::CellClear),
			"cn" | "next_cell_" => Some(Self::CellNext),
			"t" | "clock_" => Some(Self::Time),
			"m" | "mem_" => Some(Self::Memory),
			"mc" | "clear_mem" => Some(Self::MemoryClear),
			"r" | "rand_" => Some(Self::Random),
			"obs" | "obstacle" => Some(Self::Obstacle),
			"d" | "dir_" => Some(Self::Direction),
			"mov" | "moving" => Some(Self::Moving),
			"a" | "spawn_" => Some(Self::SpawnAnt),
			"kill" => Some(Self::Kill),
			"die" => Some(Self::Die),
			_ => None,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputValue {
	pub output: Peripheral,
	pub value: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeripheralBit {
	pub peripheral: Peripheral,
	pub bit: u8,
}

impl PeripheralBit {
	pub fn validate(&self, ant_type: &AntType, io_type: &IoType) -> Result<()> {
		let properties = self.peripheral.properties();

		let bit_exceeding_size = self.bit >= properties.size;

		let wrong_io_type = match properties.io_type {
			Some(req_io_type) => req_io_type != *io_type,
			None => false,
		};

		let invalid_ant_type = properties.forbidden.contains(ant_type);

		if bit_exceeding_size {
			Err(anyhow!("bit index exceeding size: {self:?}",))
		} else if wrong_io_type {
			Err(anyhow!(
				"wrong Input / Output type for Peripheral: {self:?}",
			))
		} else if invalid_ant_type {
			Err(anyhow!(
				"peripheral {:?} forbidden for {ant_type:?}",
				self.peripheral
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
