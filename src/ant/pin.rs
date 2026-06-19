#[cfg(test)]
use serde::Serialize;

#[cfg(test)]
use ts_rs::TS;

#[cfg_attr(test, derive(TS, Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
	// ## creating ants
	/// byte representing the ID of the ant, that will
	/// be spawned behind current ant, if not 0
	SpawnId,
	/// if ant is spawned by current ant,
	/// set its direction to the current ants direction plus this
	SpawnDir,
	/// if ant is spawned by current ant,
	/// set its memory to this
	SpawnMem,
	/// this plus the current layer's index will be the target layer index
	SpawnLayer,

	// ## moving ants
	/// 3 bits indicating number of 45 degrees rotations
	Dir,
	/// ant is preferred in movement / spawning conflict resolution
	Dash,
	/// current ant will not move this tick if true
	Halt,
	/// current ant will be skipped for this amount of ticks (remaining in its position)
	Wait,

	// ## removing ants
	/// kill current ant
	Die,
	/// kill ant in front of current ant, if possible
	Kill,

	// ## current tile
	/// current tile's value
	Tile,
	/// clear current tile (before writing)
	Clear,

	// ## neighboring tiles
	/// neighboring tile
	NearbyTile,
	/// true if neighboring tile contains an ant or other obstacle (i.e. border)
	NearbyAnt,
	/// neighboring ant's ID
	NearbyId,
	/// neighboring ant's Memory
	NearbyMem,

	// ## generic inputs
	/// is 1 on the birth tick (+1) of the ant, else 0
	Init,
	/// clock value incrementing each tick
	Time,
	/// clock value with bits being true every `2^(n+1)`-th tick
	Pulse,
	/// current ant's persistent memory
	Mem,
	/// 8 random bits
	Random,
	/// random bits, where each value has
	/// a chance of `1 / 2^(n+1)` of being true
	Chance,

	// ## global
	Signal,
	ExtIn,
	ExtOut,
}

#[cfg_attr(test, derive(TS, Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoType {
	Input,
	Output,
}

const BIT: u8 = 1;
const TRIPLET: u8 = 3;
const ANT_ID: u8 = BYTE;
const BYTE: u8 = 8;
const DOUBLE: u8 = 64;

#[cfg_attr(test, derive(TS, Serialize))]
#[cfg_attr(test, ts(export))]
pub struct PinDefinition {
	pub pin: Pin,
	pub code: &'static str,
	pub size: u8,
	pub io_type: Option<IoType>,
}

impl Pin {
	const PIN_DEFINITIONS: [PinDefinition; 25] = [
		PinDefinition {
			pin: Self::SpawnId,
			code: "A",
			size: ANT_ID,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::SpawnLayer,
			code: "AA",
			size: TRIPLET,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::SpawnDir,
			code: "AD",
			size: TRIPLET,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::SpawnMem,
			code: "AM",
			size: BYTE,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Tile,
			code: "C",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Clear,
			code: "CC",
			size: BIT,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Dir,
			code: "D",
			size: TRIPLET,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Dash,
			code: "DD",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Halt,
			code: "H",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Init,
			code: "J",
			size: BIT,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::ExtIn,
			code: "K",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Mem,
			code: "M",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Random,
			code: "R",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Chance,
			code: "RR",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Signal,
			code: "S",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Time,
			code: "T",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Pulse,
			code: "TT",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::NearbyAnt,
			code: "V",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::NearbyId,
			code: "VA",
			size: DOUBLE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::NearbyTile,
			code: "VC",
			size: DOUBLE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::NearbyMem,
			code: "VM",
			size: DOUBLE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Wait,
			code: "W",
			size: BYTE,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::ExtOut,
			code: "X",
			size: BYTE,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Die,
			code: "Z",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Kill,
			code: "ZZ",
			size: BIT,
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
			.find(|x| x.code == ident)
			.map(|x| x.pin)
	}

	#[inline]
	pub fn short_ident(&self) -> &'static str {
		self.definition().code
	}

	#[inline]
	/// specifies that a pin needs the line bits to be the channel bits
	/// currently only used for special treatment of `NearbyAnt`
	pub fn prefers_channel(&self) -> bool {
		matches!(self, Self::NearbyAnt)
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinValue {
	pub pin: Pin,
	pub value: u8,
}

#[cfg(test)]
mod test {
	use crate::ant::pin::IoType;

	use super::Pin;

	#[test]
	#[rustfmt::skip]
	fn export_pin_definitions() {
		println!("{}", serde_json::to_string_pretty(&Pin::PIN_DEFINITIONS).unwrap());

	}

	#[test]
	fn export_pin_stats() {
		let pin_definitions: Vec<_> = Pin::PIN_DEFINITIONS.iter().collect();

		let inputs = pin_definitions
			.iter()
			.filter(|pin| pin.io_type != Some(IoType::Output));

		let outputs = pin_definitions
			.iter()
			.filter(|pin| pin.io_type != Some(IoType::Input));

		let in_count = inputs.clone().count();

		let out_count = outputs.clone().count();

		let total_count = pin_definitions.len();

		let in_size: u16 = inputs.map(|pin| pin.size as u16).sum();
		let out_size: u16 = outputs.map(|pin| pin.size as u16).sum();
		let total_size: u16 = pin_definitions.iter().map(|pin| pin.size as u16).sum();

		println!(
			"input types: {in_count}\noutput types: {out_count}\ntotal types: {total_count}\n\ninput size: {in_size}\noutput size: {out_size}\ntotal size: {total_size}"
		);
	}
}
