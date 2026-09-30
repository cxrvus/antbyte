use crate::util::vec2::{PosOffsetTable, generate_offset_table};

#[derive(Clone)]
pub struct WorldCache {
	offset_table: PosOffsetTable,
}

impl WorldCache {
	pub fn new(euclid: bool) -> Self {
		Self {
			offset_table: generate_offset_table(euclid),
		}
	}

	#[inline]
	pub fn offset_table(&self) -> PosOffsetTable {
		self.offset_table
	}
}
