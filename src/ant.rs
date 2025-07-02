use crate::bitvec::BitVec;

pub enum Variant {
	Queen,
	Worker,
}

pub struct Ant {
	brain: Circuit,
	age: u8,
	variant: Variant,
}

pub struct Circuit {
	inputs: u8,
	layers: Vec<Layer>,
}

impl Circuit {
	pub fn tick(input: BitVec) -> BitVec {
		todo!()
	}
}

pub struct Layer {
	neurons: Vec<Neuron>,
	weights: Matrix,
	carries: BitVec,
}

pub struct Matrix(Vec<BitVec>);

pub enum Neuron {
	Or,
	And,
	Nor,
	Nand,
	Circuit(usize),
}
