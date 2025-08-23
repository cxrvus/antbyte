use anyhow::{Error, Result, anyhow};

#[derive(Clone, Debug)]
pub struct TruthTable {
	layers: Vec<Layer>,
	input_count: usize,
}

impl TruthTable {
	pub fn new(input_count: usize, layers: Vec<Layer>) -> Self {
		Self {
			input_count,
			layers,
		}
	}

	pub fn input_count(&self) -> usize {
		self.input_count
	}

	pub fn tick(&self, input: u32) -> u32 {
		let mut layer_input = input;

		for layer in &self.layers {
			layer_input = layer.tick(layer_input);
		}

		layer_input
	}
}

impl TryFrom<&str> for TruthTable {
	type Error = Error;

	fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
		Self::try_from(value.to_string())
	}
}

impl TryFrom<String> for TruthTable {
	type Error = Error;

	fn try_from(_code: String) -> Result<TruthTable> {
		todo!("rewrite using truth table notation");
	}
}

#[derive(Clone, Debug)]
pub struct Layer {
	neurons: Vec<Neuron>,
}

impl Layer {
	pub fn new(neurons: Vec<Neuron>) -> Self {
		Self { neurons }
	}

	pub fn neuron_count(&self) -> usize {
		self.neurons.len()
	}

	pub fn tick(&self, input: u32) -> u32 {
		let mut layer_output = 0;

		for neuron_index in 0..self.neuron_count() {
			let neuron = &self.neurons[neuron_index];
			let neuron_output = neuron.tick(input) as u32;

			layer_output |= neuron_output << neuron_index;
		}

		layer_output
	}
}

#[derive(Clone, Debug)]
pub struct Neuron {
	sign: u32,
	mask: u32,
}

impl Neuron {
	pub fn new(sign: u32, mask: u32) -> Self {
		Self { sign, mask }
	}

	pub fn tick(&self, value: u32) -> bool {
		((value ^ self.sign) & self.mask) != 0
	}
}

#[cfg(test)]
mod tests {
	use crate::truth_table::TruthTable;

	#[test]
	fn buf() {
		let buf = TruthTable::try_from("+").unwrap();

		assert_eq!(buf.tick(0), 0);
		assert_eq!(buf.tick(1), 1);
	}

	#[test]
	fn not() {
		let not = TruthTable::try_from("-").unwrap();

		assert_eq!(not.tick(0), 1);
		assert_eq!(not.tick(1), 0);
	}

	#[test]
	fn or() {
		let or = TruthTable::try_from("++").unwrap();

		assert_eq!(or.tick(0b00), 0);
		assert_eq!(or.tick(0b01), 1);
		assert_eq!(or.tick(0b10), 1);
		assert_eq!(or.tick(0b11), 1);

		let or3 = TruthTable::try_from("+++").unwrap();

		assert_eq!(or3.tick(0b000), 0);
		assert_eq!(or3.tick(0b010), 1);
		assert_eq!(or3.tick(0b111), 1);
	}

	#[test]
	fn and() {
		let and = TruthTable::try_from("--;;-").unwrap();

		assert_eq!(and.tick(0b00), 0);
		assert_eq!(and.tick(0b01), 0);
		assert_eq!(and.tick(0b10), 0);
		assert_eq!(and.tick(0b11), 1);

		let and3 = TruthTable::try_from("---;;-").unwrap();

		assert_eq!(and3.tick(0b000), 0);
		assert_eq!(and3.tick(0b010), 0);
		assert_eq!(and3.tick(0b111), 1);
	}

	#[test]
	fn xor() {
		let xor = TruthTable::try_from("-+;+-;;--").unwrap();

		assert_eq!(xor.tick(0b00), 0);
		assert_eq!(xor.tick(0b01), 1);
		assert_eq!(xor.tick(0b10), 1);
		assert_eq!(xor.tick(0b11), 0);
	}
}
