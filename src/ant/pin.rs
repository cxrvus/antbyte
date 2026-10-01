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
	AntId,
	/// if ant is spawned by current ant,
	/// set its rotation to the current ants direction plus this
	ChildRotation,
	/// makes ChildRotation counterclockwise
	ChildLeft,
	/// if ant is spawned by current ant,
	/// set its memory to this
	ChildMem,
	/// this plus the current layer's index will be the target layer index
	ChildLayer,

	// ## moving ants
	/// represents clockwise rotation with reverse bit order, where 256 would be a full turn
	Rotation,
	/// makes Rotation counterclockwise
	RotateLeft,
	/// resets Rotation to local birth rotation or is set to whether rotation equals birth rotation
	RotateReset,
	/// ant is preferred in movement / spawning conflict resolution
	TieBreaker,
	/// current ant will not move this tick if true
	Halt,
	/// current ant will be skipped for this amount of ticks (remaining in its position)
	Sleep,

	// ## removing ants
	/// kill current ant
	Die,
	/// kill ant in front of current ant, if possible
	Kill,

	// ## current tile
	/// current tile's value
	Color,
	/// empty current tile (before writing) or check if it's empty (if input)
	Clear,

	// ## neighboring tiles
	/// neighboring tile
	NearbyColor,
	/// true if neighboring tile contains an ant or other obstacle (i.e. border)
	Obstacle,
	/// neighboring ant's ID
	NearbyId,
	/// neighboring ant's Memory
	NearbyMem,

	// ## generic inputs
	/// is 1 on the birth tick (+1) of the ant, else 0
	Initial,
	/// current ant's persistent memory
	Mem,
	/// counter value incrementing each tick
	Counter,
	/// is set every 2^nth local tick
	Clock,
	/// 8 random bits
	Random,
	/// is set with a probability of 2^(-(n+1))
	Probability,

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
const BYTE: u8 = 8;
const WORD: u8 = 16;
const QWORD: u8 = 64;

#[cfg_attr(test, derive(TS, Serialize))]
#[cfg_attr(test, ts(export))]
pub struct PinDefinition {
	pub pin: Pin,
	pub code: &'static str,
	pub size: u8,
	pub io_type: Option<IoType>,
}

impl Pin {
	const PIN_DEFINITIONS: [PinDefinition; 28] = [
		PinDefinition {
			pin: Self::AntId,
			code: "A_ID",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::ChildLayer,
			code: "A_LYR",
			size: TRIPLET,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::ChildMem,
			code: "A_MEM",
			size: BYTE,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::ChildLeft,
			code: "A_LFT",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::ChildRotation,
			code: "A_ROT",
			size: BYTE,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Clear,
			code: "CLR",
			size: BIT,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Color,
			code: "COL",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Counter,
			code: "CTR",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Die,
			code: "DIE",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Clock,
			code: "EVR",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Halt,
			code: "HLT",
			size: BIT,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Initial,
			code: "INL",
			size: BIT,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Kill,
			code: "KLL",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::RotateLeft,
			code: "LFT",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::Mem,
			code: "MEM",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::NearbyColor,
			code: "N_COL",
			size: QWORD,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::NearbyId,
			code: "N_ID",
			size: QWORD,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::NearbyMem,
			code: "N_MEM",
			size: QWORD,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Obstacle,
			code: "OBS",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Probability,
			code: "PRB",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Random,
			code: "RND",
			size: BYTE,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::Rotation,
			code: "ROT",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::RotateReset,
			code: "RST",
			size: BIT,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Signal,
			code: "SIG",
			size: BYTE,
			io_type: None,
		},
		PinDefinition {
			pin: Self::Sleep,
			code: "SLP",
			size: BYTE,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::TieBreaker,
			code: "TBK",
			size: BIT,
			io_type: Some(IoType::Output),
		},
		PinDefinition {
			pin: Self::ExtIn,
			code: "X_IN",
			size: WORD,
			io_type: Some(IoType::Input),
		},
		PinDefinition {
			pin: Self::ExtOut,
			code: "X_OUT",
			size: WORD,
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
	/// specifies that a pin needs the line bits to be the channel bits.
	///
	/// currently only used for special treatment of `NearbyAnt`.
	pub fn prefers_channel(&self) -> bool {
		matches!(self, Self::Obstacle)
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinValue {
	pub pin: Pin,
	pub value: u16,
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
