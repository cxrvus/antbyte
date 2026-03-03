use std::cmp::Ordering;

use anyhow::{Ok, Result, anyhow, bail};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

struct MetadataRecord {
	peripheral: Peripheral,
	short: &'static str,
	aliases: &'static [&'static str],
	size: u8,
	io_type: Option<IoType>,
}

impl Peripheral {
	const METADATA: [MetadataRecord; 12] = [
		MetadataRecord {
			peripheral: Self::Cell,
			short: "C",
			aliases: &["CELL_"],
			size: CELL,
			io_type: None,
		},
		MetadataRecord {
			peripheral: Self::CellClear,
			short: "CC",
			aliases: &["CLEAR"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			peripheral: Self::CellNext,
			short: "CN",
			aliases: &["NEXT_CELL_"],
			size: CELL,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			peripheral: Self::Obstacle,
			short: "CX",
			aliases: &["OBS", "OBSTACLE"],
			size: BIT,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			peripheral: Self::Time,
			short: "T",
			aliases: &["CLOCK_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			peripheral: Self::Memory,
			short: "M",
			aliases: &["MEM_"],
			size: BYTE,
			io_type: None,
		},
		MetadataRecord {
			peripheral: Self::Random,
			short: "R",
			aliases: &["RAND_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			peripheral: Self::Direction,
			short: "D",
			aliases: &["DIR_"],
			size: DIR,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			peripheral: Self::Halted,
			short: "DX",
			aliases: &["HALT"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			peripheral: Self::SpawnAnt,
			short: "A",
			aliases: &["SPAWN_"],
			size: ANT_ID,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			peripheral: Self::Kill,
			short: "AK",
			aliases: &["KILL"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			peripheral: Self::Die,
			short: "AX",
			aliases: &["DIE"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
	];

	fn metadata(&self) -> &MetadataRecord {
		Self::METADATA
			.iter()
			.find(|m| m.peripheral == *self)
			.expect("peripheral without metadata specification")
	}

	pub fn properties(&self) -> PeripheralProperties {
		let metadata = self.metadata();

		let props = PeripheralProperties {
			size: metadata.size,
			io_type: metadata.io_type,
		};

		debug_assert_ne!(props.size, 0);

		props
	}

	pub fn from_ident(ident: &str) -> Option<Self> {
		Self::METADATA
			.iter()
			.find(|x| x.short == ident || x.aliases.contains(&ident))
			.map(|x| x.peripheral)
	}

	pub fn short_ident(&self) -> &'static str {
		self.metadata().short
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

impl Serialize for PeripheralBit {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_ident())
	}
}

impl<'de> Deserialize<'de> for PeripheralBit {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let ident = String::deserialize(deserializer)?;
		Self::from_ident(&ident).map_err(serde::de::Error::custom)
	}
}

impl PeripheralBit {
	pub fn to_ident(&self) -> String {
		let mut ident = self.peripheral.short_ident().to_owned();

		if self.peripheral.properties().size > BIT {
			ident.push_str(&format!("{:x}", self.bit));
		}

		ident
	}

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
