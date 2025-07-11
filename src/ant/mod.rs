pub mod circuit;
pub mod parser;

use circuit::Circuit;

pub enum Variant {
	Worker,
	Queen,
}

pub struct Ant {
	brain: Circuit,
	age: u8,
	variant: Variant,
}

impl Ant {
	pub fn new(circuit: Circuit) -> Self {
		Self {
			brain: circuit,
			age: 0,
			variant: Variant::Worker,
		}
	}
}

#[rustfmt::skip]
pub enum Stimulus {
	/// (bit) always true
	Constant 	= 0b00000001, 
	/// global time, cyclic
	Time 		= 0b00000010,
	/// time since spawn, cyclic
	Age 		= 0b00000100,
	/// current cell value
	Cell 		= 0b00001000,
	/// value of cell cell that is ahead
	NextCell 	= 0b00010000,
	/// (8 + 6 bits) memory value
	Memory		= 0b00100000,
	/// random byte
	Noise		= 0b01000000,
	/// (bit) is other ant ahead
	Ant			= 0b10000000,
}

#[rustfmt::skip]
pub enum Action {
	/// (8 bits)
	CellValue		= 0b00000001,
	/// (bit)
	CellWrite		= 0b00000010,
	/// (8 + 6 bits)
	MemoryValue		= 0b00000100,
	/// (bit)
	MemoryWrite		= 0b00001000,
	/// (2 bit XY + 1 to move)
	Direction		= 0b00010000,
	/// (bit)
	Die				= 0b01000000,
	/// Queen only (3 bit ID + 1 to spawn)
	Spawn			= 0b10000000,
}
