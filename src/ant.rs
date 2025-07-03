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

#[derive(Clone)]
pub struct Circuit {
	inputs: usize,
	layers: Vec<Layer>,
}

impl Circuit {
	pub fn tick(&self, input: &BitVec) -> BitVec {
		assert_eq!(input.len(), self.inputs);

		let spread_input = input
			.iter()
			.map(|&bit| BitVec::from(bit))
			.collect::<Vec<_>>();

		let mut last_layer_values = spread_input.clone();

		for layer in &self.layers {
			let mut current_layer_results: Vec<BitVec> = vec![];
			for (n, neuron) in layer.0.iter().enumerate() {
				let neuron_result = neuron.tick(&last_layer_values[n]);
				current_layer_results.push(neuron_result);
				// todo: handle carry out
			}
			last_layer_values = current_layer_results;
		}

		last_layer_values
			.iter()
			.map(|bit_vec| {
				bit_vec
					.unary()
					.expect("output layer neurons may only have unary outputs")
				// todo: validate this assertion on struct creation
			})
			.collect::<Vec<_>>()
			.into()
	}
}

#[derive(Clone)]
pub struct Layer(pub Vec<Neuron>);

#[derive(Clone)]
pub struct Neuron {
	weights: BitVec,
	function: NeuronFunction,
	// todo: add has_carry_out: bool
}

#[derive(Clone, Default)]
pub enum NeuronFunction {
	#[default]
	Or,
	And,
	Nor,
	Nand,
	Circuit(Circuit),
}

impl Neuron {
	pub fn tick(&self, input: &BitVec) -> BitVec {
		let input = match input.unary() {
			Some(enable) => input.gated_buffer(enable),
			None => match &self.function {
				NeuronFunction::Circuit(_) => self.sub_circuit_mask(input),
				_ => self.primitive_mask(input),
			},
		};

		match &self.function {
			NeuronFunction::Or => input.or_sum().into(),
			NeuronFunction::And => input.and_sum().into(),
			NeuronFunction::Nor => (!input.or_sum()).into(),
			NeuronFunction::Nand => (!input.and_sum()).into(),
			NeuronFunction::Circuit(circuit) => circuit.tick(&input),
		}
	}

	fn primitive_mask(&self, input: &BitVec) -> BitVec {
		input.and(&self.weights)
	}

	fn sub_circuit_mask(&self, input: &BitVec) -> BitVec {
		let Self { weights, .. } = &self;

		assert_eq!(input.len(), weights.len());

		input
			.iter()
			.zip(weights.iter())
			.filter_map(|(input_bit, weight)| if *weight { Some(*input_bit) } else { None })
			.collect::<Vec<_>>()
			.into()
	}
}
