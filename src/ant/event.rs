use std::cmp::Ordering;

use anyhow::{Ok, Result, anyhow, bail};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
	// ## cell interaction
	CellClear,
	Cell,
	CellNext,

	// ## universal inputs:
	Time,
	Pulse,
	Memory,
	Random,
	Chance,

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
pub struct EventProperties {
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

pub struct MetadataRecord {
	event: Event,
	short: &'static str,
	aliases: &'static [&'static str],
	size: u8,
	io_type: Option<IoType>,
}

impl Event {
	pub const METADATA: [MetadataRecord; 14] = [
		MetadataRecord {
			event: Self::Cell,
			short: "C",
			aliases: &["CELL_"],
			size: CELL,
			io_type: None,
		},
		MetadataRecord {
			event: Self::CellClear,
			short: "CC",
			aliases: &["CLEAR"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			event: Self::CellNext,
			short: "CN",
			aliases: &["NEXT_CELL_"],
			size: CELL,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			event: Self::Obstacle,
			short: "AC",
			aliases: &["OBS", "OBSTACLE"],
			size: BIT,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			event: Self::Time,
			short: "T",
			aliases: &["CLOCK_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			event: Self::Pulse,
			short: "TT",
			aliases: &["PULSE_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			event: Self::Memory,
			short: "M",
			aliases: &["MEM_"],
			size: BYTE,
			io_type: None,
		},
		MetadataRecord {
			event: Self::Random,
			short: "R",
			aliases: &["RAND_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			event: Self::Chance,
			short: "RR",
			aliases: &["CHANCE_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		MetadataRecord {
			event: Self::Direction,
			short: "D",
			aliases: &["DIR_"],
			size: DIR,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			event: Self::Halted,
			short: "DX",
			aliases: &["HALT"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			event: Self::SpawnAnt,
			short: "A",
			aliases: &["SPAWN_"],
			size: ANT_ID,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			event: Self::Kill,
			short: "AK",
			aliases: &["KILL"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		MetadataRecord {
			event: Self::Die,
			short: "AX",
			aliases: &["DIE"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
	];

	fn metadata(&self) -> &MetadataRecord {
		Self::METADATA
			.iter()
			.find(|m| m.event == *self)
			.expect("event without metadata specification")
	}

	pub fn properties(&self) -> EventProperties {
		let metadata = self.metadata();

		let props = EventProperties {
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
			.map(|x| x.event)
	}

	pub fn short_ident(&self) -> &'static str {
		self.metadata().short
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputValue {
	pub output: Event,
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

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventBit {
	pub event: Event,
	pub bit: u8,
}

impl Serialize for EventBit {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_ident())
	}
}

impl<'de> Deserialize<'de> for EventBit {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let ident = String::deserialize(deserializer)?;
		Self::from_ident(&ident).map_err(serde::de::Error::custom)
	}
}

impl EventBit {
	pub fn to_ident(&self) -> String {
		let mut ident = self.event.short_ident().to_owned();

		if self.event.properties().size > BIT {
			ident.push_str(&format!("{:x}", self.bit));
		}

		ident
	}

	pub fn validate(&self, io_type: &IoType) -> Result<()> {
		let properties = self.event.properties();

		let bit_exceeding_size = self.bit >= properties.size;

		let wrong_io_type = match properties.io_type {
			Some(req_io_type) => req_io_type != *io_type,
			None => false,
		};

		if bit_exceeding_size {
			Err(anyhow!("bit index exceeding size: {self:?}",))
		} else if wrong_io_type {
			Err(anyhow!("wrong Input / Output type for Event: {self:?}",))
		} else {
			Ok(())
		}
	}

	const EVENT_PTN: &str = r"^([A-Z_]+)([0-8])?$";

	pub fn from_ident(ident: &str) -> Result<Self> {
		let re = Regex::new(Self::EVENT_PTN).unwrap();

		let captures = re
			.captures(ident)
			.ok_or(anyhow!("'{ident}' is not a valid event"))?;

		let event_ident = captures.get(1).unwrap().as_str();

		let event = Event::from_ident(event_ident)
			.ok_or(anyhow!("'{event_ident}' is not a valid event type"))?;

		let bit_index = captures
			.get(2)
			.map(|m| u8::from_str_radix(m.as_str(), 16).unwrap());

		let size = event.properties().size;

		if let Some(bit_index) = bit_index {
			if size == 1 {
				bail!("may not have a bit index in one-bit events\n(in '{ident}')");
			} else if bit_index >= size {
				bail!(
					"bit index may not exceed event bit capacity:\n{bit_index} >= {size}\n(in '{ident}')"
				);
			}
		};

		Ok(Self {
			event,
			bit: bit_index.unwrap_or_default(),
		})
	}
}

#[cfg(test)]
mod test {
	use super::{Event, IoType, MetadataRecord};
	use IoType::*;

	#[test]
	#[rustfmt::skip]
	fn show_metadata() {
		let entries = Event::METADATA;

		let inputs = entries.iter().filter(|x| x.io_type == Some(Input)).collect::<Vec<_>>();
		let outputs = entries.iter().filter(|x| x.io_type == Some(Output)).collect::<Vec<_>>();

		// number literals accounting for special events (C and M) and planned events (AD, K, X)...

		let input_count = inputs.len() + 2 + 1;
		let output_count = outputs.len() + 2 + 2;

		let input_size = inputs.iter().map(|x| x.size).sum::<u8>() + 4 + 8 + 8;
		let output_size = outputs.iter().map(|x| x.size).sum::<u8>() + 8 + 4 + 8 + 3;

		println!("input count: {input_count}");
		println!("output count: {output_count}");
		println!("total count: {}", input_count + output_count);

		println!();

		println!("input size: {input_size}");
		println!("output size: {output_size}");
		println!("total size: {}", input_size + output_size);

		println!();
		println!();

		for entry in entries {
			let MetadataRecord { short, size, io_type, ..  } = entry;

			let io_type = match io_type {
				None => "IO",
				Some(Input) => "I",
				Some(Output) => "*",
			};

			println!("{short}; {size}; {io_type}")
		}
	}
}
