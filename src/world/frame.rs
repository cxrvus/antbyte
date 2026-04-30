use std::collections::BTreeMap;

use crate::util::vec2::Position;

pub struct FrameInput {
	pub keys: u8,
}

pub struct FrameOutput {
	pub fg: BTreeMap<Position, u8>,
	pub bg: BTreeMap<Position, u8>,
}
