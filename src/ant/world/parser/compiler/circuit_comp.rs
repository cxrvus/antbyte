use std::collections::HashMap;

use anyhow::{Result, anyhow};

use super::{FlatAssignment, FlatCircuit};

use crate::{
	ant::{
		compiler::{Node, Normalizer},
		world::parser::{CircuitType, ParsedCircuit},
	},
	util::find_dupe,
};

pub(super) fn flatten_circuits(
	parsed_circuits: Vec<ParsedCircuit>,
) -> Result<HashMap<String, FlatCircuit>> {
	let mut normalizer = Normalizer::default();

	for circuit in parsed_circuits.into_iter() {
		validate_circuit_io(&circuit)?;

		let circuit_name = circuit.name.clone();
		let flat_circuit = normalizer.flatten_circuit(circuit)?;

		if normalizer
			.0
			.insert(circuit_name.clone(), flat_circuit)
			.is_some()
		{
			return Err(anyhow!("circuit name '{circuit_name}' used more than once"));
		}
	}

	Ok(normalizer.0)
}

impl Normalizer {
	fn flatten_circuit(&self, circuit: ParsedCircuit) -> Result<FlatCircuit> {
		let mut exp_index = 0;
		let mut func_index = 0;
		let mut nodes: Vec<Node> = vec![];

		for assignment in circuit.assignments.iter() {
			exp_index += 1;

			let mut sub_assignments = assignment.flatten(&mut exp_index);

			resolve_and_gates(&mut sub_assignments);

			self.validate_assignments(&sub_assignments, &circuit)?;

			for sub_assignment in sub_assignments {
				match sub_assignment.call.as_str() {
					"or" => {
						if sub_assignment.assignees.len() != 1 {
							return Err(anyhow!(
								"the result of an OR may only be assigned to a single assignee"
							));
						}

						nodes.push(sub_assignment.into());
					}
					call => {
						func_index += 1;
						let expanded = self.expand_func_call(call, &sub_assignment, func_index)?;
						nodes.extend(expanded);
					}
				}
			}

			println!("\n\n\n") //TODO: remove (dbg)
		}

		let all_assignees: Vec<_> = nodes.iter().map(|node| &node.ident).collect();

		if let Some(dupe_assignee) = find_dupe(&all_assignees) {
			return Err(anyhow!(
				"identifier '{dupe_assignee}' can not be assigned to more than once"
			));
		}

		dbg!(&nodes);

		Ok(FlatCircuit {
			nodes,
			original: circuit,
		})
	}

	fn expand_func_call(
		&self,
		call: &str,
		assignment: &FlatAssignment,
		func_index: u32,
	) -> Result<Vec<Node>> {
		let func = self
			.0
			.get(call)
			.ok_or(anyhow!("unknown function: '{call}'"))?;

		if let CircuitType::Ant(ant_type) = &func.original.circuit_type {
			return Err(anyhow!(
				"circuit '{call}' is a {ant_type:?}, not a function"
			));
		}

		validate_call_signature(func, assignment)?;

		let var_prefix = format!("_{call}{func_index:02}");

		let mut expanded_nodes = vec![];

		for mut node in func.nodes.clone() {
			if let Some(output_index) = func
				.original
				.outputs
				.iter()
				.position(|output| *output == node.ident)
			{
				// assignee represents a function output
				node.ident = assignment.assignees[output_index].clone();
			} else {
				// assignee represents a variable
				node.ident = var_prefix.clone() + &node.ident;
			}

			for node_wire in node.wires.iter_mut() {
				if let Some(func_param_index) = func
					.original
					.inputs
					.iter()
					.position(|input| *input == node_wire.target)
				{
					// wire targets a function input
					let input_wire = &assignment.wires[func_param_index];
					node_wire.target = input_wire.target.clone();
					node_wire.sign ^= input_wire.sign;
				} else {
					// wire targets a variable
					node_wire.target = var_prefix.clone() + &node_wire.target;
				}
			}

			expanded_nodes.push(node);
		}

		Ok(expanded_nodes)
	}
}

fn validate_circuit_io(circuit: &ParsedCircuit) -> Result<()> {
	if let Some(dupe_ident) = circuit
		.inputs
		.iter()
		.find(|input| circuit.outputs.iter().any(|output| output == *input))
	{
		Err(anyhow!(
			"identifier '{dupe_ident}' used as both input and output"
		))
	} else {
		Ok(())
	}
}

fn validate_call_signature(func: &FlatCircuit, assignment: &FlatAssignment) -> Result<()> {
	let (input_count, output_count, parameter_count, assignee_count) = (
		func.original.inputs.len(),
		func.original.outputs.len(),
		assignment.wires.len(),
		assignment.assignees.len(),
	);

	let func_name = &func.original.name;

	if parameter_count != input_count {
		Err(anyhow!(
			"function '{func_name}' has been given an invalid number of parameter values\nexpected {input_count}, got {parameter_count}"
		))
	} else if assignee_count != output_count {
		Err(anyhow!(
			"function '{func_name}' has been given an invalid number of assignees\nexpected {output_count}, got {assignee_count}"
		))
	} else {
		Ok(())
	}
}

/// transform AND into OR ([DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws))
fn resolve_and_gates(assignments: &mut [FlatAssignment]) {
	assignments
		.iter_mut()
		.filter(|assignment| assignment.call == "and")
		.for_each(|assignment| {
			assignment
				.wires
				.iter_mut()
				.for_each(|wire| wire.sign = !wire.sign);

			assignment.sign = !assignment.sign;
			assignment.call = "or".into();
		});
}
