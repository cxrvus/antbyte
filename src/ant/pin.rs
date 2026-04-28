#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
	// ## cell interaction
	/// clear current cell (before writing)
	Clear,
	/// read or write selected bits of current cell
	Cell,
	/// read cell in front of current ant
	Next,

	// ## universal inputs:
	/// clock value incrementing each tick
	Time,
	/// clock value with bits being true every `2^(n+1)`-th tick
	Pulse,
	/// read or write the current ant's persistent memory
	Mem,
	/// 8 random bits
	Random,
	/// random bits, where each value has
	/// a chance of `1 / 2^(n+1)` of being true
	Chance,

	// ## ant interaction inputs
	/// true if cell in front current of ant contains an ant
	Collide,
	/// kill ant in front of current ant, if possible
	Kill,

	// ## ant interaction outputs
	/// 3 bits indicating number of 45 degrees rotations
	Dir,
	/// current ant will not move this tick if true
	Halt,

	/// if ant is spawned by current ant,
	/// set its direction to the current ants direction plus this
	AntDir,
	/// if ant is spawned by current ant,
	/// set its memory to this
	AntMem,
	/// byte representing the ID of the ant, that will
	/// be spawned behind current ant, if not 0
	AntSpawn,

	Signal,

	// ## external
	ExtIn,
	ExtOut,

	// ## Die
	Die,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoType {
	Input,
	Output,
}

const BIT: u8 = 1;
const DIR: u8 = 3;
const ANT_ID: u8 = BYTE;
const BYTE: u8 = 8;

pub struct PinDefinition {
	pub pin: Pin,
	pub short: &'static str,
	pub aliases: &'static [&'static str],
	pub size: u8,
	pub io_type: Option<IoType>,
}

impl Pin {
	const PIN_DEFINITIONS: [PinDefinition; 19] = [
		PinDefinition {
			pin: Self::Cell,
			short: "C",
			aliases: &["CELL_"],
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Clear,
			short: "CC",
			aliases: &["CLEAR"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Next,
			short: "CN",
			aliases: &["NEXT_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Collide,
			short: "AC",
			aliases: &["COLLIDE"],
			size: BIT,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Time,
			short: "T",
			aliases: &["TIME_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Pulse,
			short: "TT",
			aliases: &["PULSE_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Mem,
			short: "M",
			aliases: &["MEM_"],
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Random,
			short: "R",
			aliases: &["RAND_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Chance,
			short: "RR",
			aliases: &["CHANCE_"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Dir,
			short: "D",
			aliases: &["DIR_"],
			size: DIR,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Halt,
			short: "DX",
			aliases: &["HALT"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::AntSpawn,
			short: "A",
			aliases: &["ANT_"],
			size: ANT_ID,
			io_type: None,
		},
		PinDefinition {
			pin: Self::AntDir,
			short: "AD",
			aliases: &["ANT_DIR_"],
			size: DIR,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::AntMem,
			short: "AM",
			aliases: &["ANT_MEM_"],
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Kill,
			short: "AK",
			aliases: &["KILL"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Signal,
			short: "S",
			aliases: &["SIGNAL_"],
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Die,
			short: "AX",
			aliases: &["DIE"],
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::ExtIn,
			short: "K",
			aliases: &["INPUT", "KEY"],
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::ExtOut,
			short: "X",
			aliases: &["OUTPUT", "AUDIO"],
			size: BYTE,
			io_type: Some(IoType::Output),
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

		println!("SHORT; ALIAS; SIZE; IO_TYPE;");

		for entry in entries {
			let PinDefinition { short, aliases, size, io_type, .. } = entry;

			let io_type = match io_type {
				None => "*",
				Some(Input) => "I",
				Some(Output) => "O",
			};

			let alias = aliases[0];

			println!("{short}; {alias}; {size}; {io_type};")
		}
	}
}
