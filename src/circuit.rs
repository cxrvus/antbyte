use crate::{bitvec::BitVec, matrix::Matrix};

#[derive(Clone)]
pub struct Circuit {
	inputs: usize,
	layers: Vec<Layer>,
}

impl Circuit {
	pub fn tick(&self, input: &BitVec) -> BitVec {
		assert_eq!(input.len(), self.inputs);

		let mut layer_input = input.clone();

		for layer in &self.layers {
			layer_input = layer.tick(&layer_input);
		}

		layer_input
	}
}

#[derive(Clone)]
pub struct Layer {
	neuron_count: usize,
	weights: Matrix<Weight>,
}

impl Layer {
	pub fn tick(&self, input: &BitVec) -> BitVec {
		let mut layer_output = BitVec::new();

		for neuron_index in 0..self.neuron_count {
			// todo: get current neuron weights
			// todo: apply to input (XOR bit0, AND bit1)
			// todo: OR-sum

			let neuron_output = todo!();
			layer_output.push(neuron_output);
			// idea: implement carry out
		}

		layer_output
	}
}

#[derive(Clone)]
pub enum Weight {
	Zero = 0,
	Pos = 1,
	Neg = 3,
}

impl Weight {
	pub fn flip(&mut self) {
		*self = match self {
			Self::Zero => Self::Zero,
			Self::Pos => Self::Neg,
			Self::Neg => Self::Pos,
		};
	}

	pub fn apply(&self, input: BitVec) -> BitVec {
		todo!()
	}
}
