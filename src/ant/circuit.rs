use crate::util::{bitvec::BitVec, matrix::Matrix};

#[derive(Clone)]
pub struct Circuit {
	layers: Vec<Layer>,
}

impl Circuit {
	pub fn new(layers: Vec<Layer>) -> Self {
		// todo: assert correct weight dimensions in layers

		Self { layers }
	}

	pub fn input_count(&self) -> usize {
		self.layers[0].weights.width
	}

	pub fn tick(&self, input: &BitVec) -> BitVec {
		assert_eq!(input.len(), self.input_count());

		let mut layer_input = input.clone();

		for layer in &self.layers {
			layer_input = layer.tick(&layer_input);
		}

		layer_input
	}
}

#[derive(Clone)]
pub struct Layer {
	weights: Matrix<Weight>,
}

impl Layer {
	pub fn new(weights: Matrix<Weight>) -> Self {
		Self { weights }
	}

	pub fn neuron_count(&self) -> usize {
		self.weights.height
	}

	pub fn tick(&self, input: &BitVec) -> BitVec {
		let mut layer_output = BitVec::new();

		for neuron_index in 0..self.neuron_count() {
			let neuron_weights = self.weights.get_row(neuron_index).unwrap().clone();
			assert_eq!(neuron_weights.len(), input.len());

			let weighted_input = Self::apply_weights(input, neuron_weights);
			let neuron_output = weighted_input.or_sum();
			layer_output.push(neuron_output);
		}

		layer_output
	}

	fn apply_weights(input: &BitVec, weights: Vec<&Weight>) -> BitVec {
		// todo: optimize using bitwise XOR and AND

		let mut output = BitVec::new();

		for (i, input_bit) in input.iter().enumerate() {
			let weight = weights[i];

			let output_bit = match (weight, input_bit) {
				(Weight::Zero, _) => false,
				(Weight::Pos, true) => true,
				(Weight::Pos, false) => false,
				(Weight::Neg, true) => false,
				(Weight::Neg, false) => true,
			};

			output.push(output_bit);
		}

		output
	}
}

#[derive(Clone, Debug, Default)]
pub enum Weight {
	#[default]
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
}

pub fn string_to_weights(weight_str: String) -> Vec<Weight> {
	weight_str
		.chars()
		.map(|c| match c {
			'.' => Weight::Zero,
			'+' => Weight::Pos,
			'-' => Weight::Neg,
			other => panic!("unknown weight symbol: {other}"),
		})
		.collect()
}

// todo: add tests
#[cfg(test)]
mod tests {
	use crate::{
		ant::circuit::{Circuit, Layer, string_to_weights},
		util::{bitvec::BitVec, matrix::ProxyMatrix},
	};

	#[test]
	fn or() {
		let circuit = Circuit {
			inputs: 2,
			layers: vec![Layer {
				size: 1,
				weights: ProxyMatrix {
					width: 2,
					height: 1,
					string: "++".into(),
				}
				.convert(string_to_weights),
			}],
		};

		assert_eq!(circuit.tick(&BitVec::from(vec![false, false])), 0u8.into())
	}
}
