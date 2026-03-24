use std::cmp::Ordering;

use anyhow::{Ok, Result, anyhow, bail};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pin {
	// ## cell interaction
	Clear,
	Cell,
	NextCell,

	// ## universal inputs:
	Time,
	Pulse,
	Mem,
	Random,
	Chance,

	// ## ant interaction inputs
	See,
	Kill,

	// ## ant interaction outputs
	/// 3 bits indicating number of 45 degrees rotations
	Dir,
	Halt,

	AntDir,
	AntMem,
	Ant,

	Event,
	Send,

	Die,

	// ## external
	ExtIn,
	ExtOut,
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

pub struct PinDefinition {
	pub pin: Pin,
	pub short: &'static str,
	pub aliases: &'static [&'static str],
	pub size: u8,
	pub io_type: Option<IoType>,
	pub queen: bool,
}

impl Pin {
	pub const PIN_DEFINITIONS: [PinDefinition; 20] = [
		PinDefinition {
			pin: Self::Cell,
			short: "C",
			aliases: &["CELL_"],
			size: CELL,
			io_type: None,
			queen: false,
		},
		PinDefinition {
			pin: Self::Clear,
			short: "CC",
			aliases: &["CLEAR"],
			size: BIT,
			io_type: Some(IoType::Output),
			queen: false,
		},
		PinDefinition {
			pin: Self::NextCell,
			short: "CN",
			aliases: &["NEXT_CELL_"],
			size: CELL,
			io_type: Some(IoType::Input),
			queen: false,
		},
		PinDefinition {
			pin: Self::See,
			short: "AC",
			aliases: &["ANT_SEE"],
			size: BIT,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::Time,
			short: "T",
			aliases: &["TIME_"],
			size: BYTE,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::Pulse,
			short: "TT",
			aliases: &["PULSE_"],
			size: BYTE,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::Mem,
			short: "M",
			aliases: &["MEM_"],
			size: BYTE,
			io_type: None,
			queen: true,
		},
		PinDefinition {
			pin: Self::Random,
			short: "R",
			aliases: &["RAND_"],
			size: BYTE,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::Chance,
			short: "RR",
			aliases: &["CHANCE_"],
			size: BYTE,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::Dir,
			short: "D",
			aliases: &["DIR_"],
			size: DIR,
			io_type: Some(IoType::Output),
			queen: false,
		},
		PinDefinition {
			pin: Self::Halt,
			short: "DX",
			aliases: &["HALT"],
			size: BIT,
			io_type: Some(IoType::Output),
			queen: false,
		},
		PinDefinition {
			pin: Self::Ant,
			short: "A",
			aliases: &["ANT_"],
			size: ANT_ID,
			io_type: Some(IoType::Output),
			queen: true,
		},
		PinDefinition {
			pin: Self::AntDir,
			short: "AD",
			aliases: &["ANT_DIR_"],
			size: DIR,
			io_type: Some(IoType::Output),
			queen: true,
		},
		PinDefinition {
			pin: Self::AntMem,
			short: "AM",
			aliases: &["ANT_MEM_"],
			size: BYTE,
			io_type: Some(IoType::Output),
			queen: true,
		},
		PinDefinition {
			pin: Self::Kill,
			short: "AK",
			aliases: &["KILL"],
			size: BIT,
			io_type: Some(IoType::Output),
			queen: false,
		},
		PinDefinition {
			pin: Self::Event,
			short: "E",
			aliases: &["EVENT_"],
			size: BYTE,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::Send,
			short: "ES",
			aliases: &["SEND_"],
			size: BYTE,
			io_type: Some(IoType::Output),
			queen: true,
		},
		PinDefinition {
			pin: Self::Die,
			short: "AX",
			aliases: &["DIE"],
			size: BIT,
			io_type: Some(IoType::Output),
			queen: true,
		},
		PinDefinition {
			pin: Self::ExtIn,
			short: "K",
			aliases: &["INPUT", "KEY"],
			size: BYTE,
			io_type: Some(IoType::Input),
			queen: true,
		},
		PinDefinition {
			pin: Self::ExtOut,
			short: "X",
			aliases: &["OUTPUT", "AUDIO"],
			size: BYTE,
			io_type: Some(IoType::Output),
			queen: true,
		},
	];

	pub fn definition(&self) -> &PinDefinition {
		Self::PIN_DEFINITIONS
			.iter()
			.find(|m| m.pin == *self)
			.expect("pin without pin definition")
	}

	pub fn from_ident(ident: &str) -> Option<Self> {
		Self::PIN_DEFINITIONS
			.iter()
			.find(|x| x.short == ident || x.aliases.contains(&ident))
			.map(|x| x.pin)
	}

	pub fn short_ident(&self) -> &'static str {
		self.definition().short
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinValue {
	pub pin: Pin,
	pub value: u8,
}

impl PartialOrd for PinValue {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for PinValue {
	fn cmp(&self, other: &Self) -> Ordering {
		self.pin.cmp(&other.pin).then(self.value.cmp(&other.value))
	}
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubPin {
	pub pin: Pin,
	pub line: u8,
}

impl Serialize for SubPin {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_ident())
	}
}

impl<'de> Deserialize<'de> for SubPin {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let ident = String::deserialize(deserializer)?;
		Self::from_ident(&ident).map_err(serde::de::Error::custom)
	}
}

impl SubPin {
	pub fn to_ident(&self) -> String {
		let mut ident = self.pin.short_ident().to_owned();

		if self.pin.definition().size > BIT {
			ident.push_str(&format!("{:x}", self.line));
		}

		ident
	}

	pub fn validate(&self, io_type: &IoType) -> Result<()> {
		let definition = self.pin.definition();

		let bit_exceeding_size = self.line >= definition.size;

		let wrong_io_type = match definition.io_type {
			Some(req_io_type) => req_io_type != *io_type,
			None => false,
		};

		if bit_exceeding_size {
			Err(anyhow!("bit index exceeding size: {self:?}",))
		} else if wrong_io_type {
			Err(anyhow!("wrong Input / Output type for Pin: {self:?}",))
		} else {
			Ok(())
		}
	}

	const PIN_PTN: &str = r"^([A-Z_]+)([0-8])?$";

	pub fn from_ident(ident: &str) -> Result<Self> {
		let re = Regex::new(Self::PIN_PTN).unwrap();

		let captures = re
			.captures(ident)
			.ok_or(anyhow!("'{ident}' is not a valid pin"))?;

		let pin_ident = captures.get(1).unwrap().as_str();

		let pin =
			Pin::from_ident(pin_ident).ok_or(anyhow!("'{pin_ident}' is not a valid pin type"))?;

		let bit_index = captures
			.get(2)
			.map(|m| u8::from_str_radix(m.as_str(), 16).unwrap());

		let size = pin.definition().size;

		if let Some(bit_index) = bit_index {
			if size == 1 {
				bail!("may not have a bit index in one-bit pins\n(in '{ident}')");
			} else if bit_index >= size {
				bail!(
					"bit index may not exceed pin bit capacity:\n{bit_index} >= {size}\n(in '{ident}')"
				);
			}
		};

		Ok(Self {
			pin,
			line: bit_index.unwrap_or_default(),
		})
	}
}

#[cfg(test)]
mod test {
	use super::{IoType, Pin, PinDefinition};
	use IoType::*;

	#[test]
	#[rustfmt::skip]
	fn print_pin_definitions() {
		let entries = Pin::PIN_DEFINITIONS;

		let inputs = entries.iter().filter(|x| x.io_type == Some(Input)).collect::<Vec<_>>();
		let outputs = entries.iter().filter(|x| x.io_type == Some(Output)).collect::<Vec<_>>();

		// number literals accounting for special pins (C and M) and planned pins (none) ...

		let input_count = inputs.len() + 2;
		let output_count = outputs.len() + 2;

		let input_size = inputs.iter().map(|x| x.size).sum::<u8>() + (8 + 8) ;
		let output_size = outputs.iter().map(|x| x.size).sum::<u8>() + (8 + 8);

		println!("input count: {input_count}");
		println!("output count: {output_count}");
		println!("total count: {}", input_count + output_count);

		println!();

		println!("input size: {input_size}");
		println!("output size: {output_size}");
		println!("total size: {}", input_size + output_size);

		println!();
		println!();

		println!("SHORT; ALIAS; SIZE; IO_TYPE; QUEEN;");

		for entry in entries {
			let PinDefinition { short, aliases, size, io_type, queen, .. } = entry;

			let io_type = match io_type {
				None => "*",
				Some(Input) => "I",
				Some(Output) => "O",
			};

			let queen = match queen {
				true => "Q",
				false => "*",
			};

			let alias = aliases[0];

			println!("{short}; {alias}; {size}; {io_type}; {queen};")
		}
	}
}
