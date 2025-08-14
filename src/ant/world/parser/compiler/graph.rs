use anyhow::{Result, anyhow};

use crate::{
	ant::{
		compiler::{Graph, Node},
		peripherals::{Input, Output, PeripheralSet},
		world::parser::ParsedCircuit,
	},
	circuit::Circuit,
};

pub(super) fn create_graph(circuit: ParsedCircuit, nodes: &Vec<Node>) -> Result<Graph> {
	let used_inputs = circuit
		.inputs
		.into_iter()
		.map(Input::from_ident)
		.collect::<Result<Vec<_>>>()?;

	let input_spec = PeripheralSet::from_used(used_inputs, true)?;

	let used_outputs = circuit
		.outputs
		.into_iter()
		.map(Output::from_ident)
		.collect::<Result<Vec<_>>>()?;

	let output_spec = PeripheralSet::from_used(used_outputs, true)?;

	// TODO: fill peripherals with sorted inputs from specs, instead of unsorted user-given peripherals
	todo!()
}

impl From<Graph> for Circuit {
	fn from(value: Graph) -> Self {
		todo!()
	}
}
