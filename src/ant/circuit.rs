#[derive(Clone, Debug)]
pub struct Circuit {
	layers: Vec<Layer>,
	input_count: usize,
}

impl Circuit {
	pub fn new(input_count: usize, layers: Vec<Layer>) -> Self {
		// todo: assert correct wire dimensions in layers

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

#[derive(Clone, Debug)]
pub struct Layer {
	wires: Vec<WireArray>,
}

impl Layer {
	pub fn new(wires: Vec<WireArray>) -> Self {
		Self { wires }
	}

	pub fn neuron_count(&self) -> usize {
		self.wires.len()
	}

	pub fn tick(&self, input: u32) -> u32 {
		let mut layer_output = 0;

		for neuron_index in 0..self.neuron_count() {
			let neuron_wires = &self.wires[neuron_index];
			let wired_input = neuron_wires.apply(input);
			let neuron_output = (wired_input != 0) as u32;

			layer_output |= neuron_output << neuron_index;
		}

		layer_output
	}
}

#[derive(Clone, Debug)]
pub struct WireArray {
	inversion: u32,
	mask: u32,
}

impl WireArray {
	pub fn new(inversion: u32, mask: u32) -> Self {
		// todo: validate
		Self { inversion, mask }
	}

	pub fn apply(&self, value: u32) -> u32 {
		(value ^ self.inversion) & self.mask
	}
}

#[cfg(test)]
mod tests {
	use crate::ant::parser::Parser;

	#[test]
	fn buf() {
		let buf = Parser::parse("+".into()).unwrap();

		assert_eq!(buf.tick(0), 0);
		assert_eq!(buf.tick(1), 1);
	}

	#[test]
	fn not() {
		let not = Parser::parse("-".into()).unwrap();

		assert_eq!(not.tick(0), 1);
		assert_eq!(not.tick(1), 0);
	}

	#[test]
	fn or() {
		let or = Parser::parse("++".into()).unwrap();

		assert_eq!(or.tick(0b00), 0);
		assert_eq!(or.tick(0b01), 1);
		assert_eq!(or.tick(0b10), 1);
		assert_eq!(or.tick(0b11), 1);

		let or3 = Parser::parse("+++".into()).unwrap();

		assert_eq!(or3.tick(0b000), 0);
		assert_eq!(or3.tick(0b010), 1);
		assert_eq!(or3.tick(0b111), 1);
	}

	#[test]
	fn and() {
		let and = Parser::parse("--;;-".into()).unwrap();

		assert_eq!(and.tick(0b00), 0);
		assert_eq!(and.tick(0b01), 0);
		assert_eq!(and.tick(0b10), 0);
		assert_eq!(and.tick(0b11), 1);

		let and3 = Parser::parse("---;;-".into()).unwrap();

		assert_eq!(and3.tick(0b000), 0);
		assert_eq!(and3.tick(0b010), 0);
		assert_eq!(and3.tick(0b111), 1);
	}

	#[test]
	fn xor() {
		let xor = Parser::parse("-+;+-;;--".into()).unwrap();

		assert_eq!(xor.tick(0b00), 0);
		assert_eq!(xor.tick(0b01), 1);
		assert_eq!(xor.tick(0b10), 1);
		assert_eq!(xor.tick(0b11), 0);
	}
}
