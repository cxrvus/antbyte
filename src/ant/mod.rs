pub mod circuit;
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
