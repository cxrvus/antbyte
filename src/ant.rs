use crate::bitvec::BitVec;

pub struct Ant {
	brain: Network,
	age: u8,
	is_queen: bool,
}

pub struct Network {
	inputs: u8,
	outputs: u8,
	input_weights: Weights,
	hidden_layers: Vec<HiddenLayer>,
}

pub struct HiddenLayer {
	neurons: Vec<Neuron>,
	weights: Weights,
	carries: BitVec,
}

pub struct Weights(Vec<BitVec>);

pub enum Neuron {
	Or,
	And,
	Nor,
	Nand,
	Network(usize),
}
