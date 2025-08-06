use std::collections::HashMap;

use super::{Assignment, CircuitType, Expression, ParsedCircuit, Parser, Statement, Token};

use crate::{
	ant::{
		Archetype,
		peripherals::{Input, Output, PeripheralSet},
		world::WorldConfig,
	},
	circuit::{self, Circuit},
};

use anyhow::{Ok, Result, anyhow};

struct Graph(Vec<GraphLayer>);
struct GraphLayer(Vec<Node>);
struct Node {
	sign: bool,
	wires: Vec<u32>,
}

pub fn compile(code: String) -> Result<WorldConfig> {
	let parsed_world = Parser::new(code).parse_world()?;

	let mut config = WorldConfig::default();
	let mut parsed_circuits: Vec<ParsedCircuit> = vec![];

	for statement in parsed_world.statements {
		match statement {
			Statement::Set(key, value) => set_setting(&mut config, key, value)?,
			Statement::Declare(circuit) => {
				parsed_circuits.push(circuit);
			}
		}
	}

	let mut flattened_circuits: HashMap<String, FlattenedCircuit> = HashMap::new();

	for circuit in parsed_circuits.into_iter() {
		validate_circuit_io(&circuit)?;

		let circuit_name = circuit.name.clone();
		let flattened_circuit = flatten_circuit(circuit, &flattened_circuits)?;

		if flattened_circuits
			.insert(circuit_name.clone(), flattened_circuit)
			.is_some()
		{
			return Err(anyhow!("circuit name '{circuit_name}' used more than once"));
		}
	}

	dbg!(
		&flattened_circuits
			.iter()
			.map(|x| &x.1.assignments)
			.collect::<Vec<_>>()
	);

	// create Archetypes
	for flattened_circuit in flattened_circuits {
		let circuit = flattened_circuit.1.original;

		if let CircuitType::Ant(ant_type) = circuit.circuit_type {
			let used_inputs = circuit
				.used_inputs
				.into_iter()
				.map(Input::from_ident)
				.collect::<Result<Vec<_>>>()?;

			let input_spec = PeripheralSet::from_used(used_inputs, true)?;

			let used_outputs = circuit
				.used_outputs
				.into_iter()
				.map(Output::from_ident)
				.collect::<Result<Vec<_>>>()?;

			let output_spec = PeripheralSet::from_used(used_outputs, true)?;

			// TODO: fill peripherals with sorted inputs from specs, instead of unsorted user-given peripherals

			let archetype = Archetype {
				ant_type,
				circuit: todo!(),
				outputs: output_spec,
				inputs: input_spec,
			};

			config.archetypes.push(archetype);
		};
	}

	// dbg!(&config);

	Ok(config)
}

fn validate_circuit_io(circuit: &ParsedCircuit) -> Result<()> {
	if let Some(double_use) = circuit
		.used_inputs
		.iter()
		.find(|input| circuit.used_outputs.iter().any(|output| output == *input))
	{
		Err(anyhow!(
			"identifier '{double_use}' used as both input and output"
		))
	} else {
		Ok(())
	}
}

#[derive(Debug)]
struct FlattenedCircuit {
	original: ParsedCircuit,
	assignments: Vec<FlattenedAssignment>,
}

#[derive(Debug)]
struct FlattenedAssignment {
	lhs: String,
	sign: bool,
	wires: Vec<Wire>,
}

#[derive(Debug)]
struct Wire {
	sign: bool,
	target: String,
}

fn flatten_circuit(
	circuit: ParsedCircuit,
	flattened_circuits: &HashMap<String, FlattenedCircuit>,
) -> Result<FlattenedCircuit> {
	let mut flattened_assignments: Vec<FlattenedAssignment> = vec![];

	let ParsedCircuit {
		name: circuit_name,
		circuit_type,
		used_inputs: inputs,
		used_outputs: outputs,
		assignments,
	} = &circuit;

	let mut exp_counter = 0;

	for Assignment {
		lhs: assignees,
		rhs: expression,
	} in assignments.iter()
	{
		let mut assignment_stack: Vec<FlattenedAssignment> = vec![];
		let mut exp_stack: Vec<&Expression> = vec![expression];

		// depth-first traversal
		while let Some(exp) = exp_stack.pop() {
			let Expression {
				ident,
				sign,
				parameters,
			} = exp;

			// if it's a function call
			if let Some(parameters) = parameters {
				assignment_stack.push(FlattenedAssignment {
					lhs: format!("call_{ident}_{exp_counter:03}"),
					sign: *sign,
					wires: vec![],
				});

				parameters
					.iter()
					.rev()
					.for_each(|parameter| exp_stack.push(parameter));
			} else {
				if !inputs.contains(ident) && !flattened_assignments.iter().any(|x| x.lhs == *ident)
				{
					return if flattened_circuits.contains_key(ident) {
						Err(anyhow!("'{ident}' is a circuit, not an input"))
					} else if outputs.contains(ident) {
						Err(anyhow!("'{ident}' is an output, not an input"))
					} else {
						Err(anyhow!("unknown identifier: '{ident}'"))
					};
				}

				if let Some(mut current_assignment) = assignment_stack.pop() {
					current_assignment.wires.push(Wire {
						sign: *sign,
						target: ident.clone(),
					});
					assignment_stack.push(current_assignment);
				}
			}

			exp_counter += 1;
		}

		// Move all completed assignments from stack to final result
		flattened_assignments.extend(assignment_stack);
	}

	Ok(FlattenedCircuit {
		assignments: flattened_assignments,
		original: circuit,
	})
}

fn set_setting(config: &mut WorldConfig, key: String, value: Token) -> Result<()> {
	let key = key.to_ascii_lowercase();

	// todo: implement all WorldConfig properties
	// idea: more elegant match statement
	match key.as_str() {
		key @ "width" | key @ "height" => {
			if let Token::Number(number) = value {
				*match key {
					"width" => &mut config.width,
					"height" => &mut config.height,
					_ => unreachable!(),
				} = number as usize;
				Ok(())
			} else {
				invalid_type(&value, "number (pixel count)", key)
			}
		}
		other => Err(anyhow!("unknown setting: {}", other)),
	}
}

fn invalid_type(actual: &Token, expected: &str, key: &str) -> Result<()> {
	Err(anyhow!(
		"expected {expected}, got {actual:?}\nfor key {key}"
	))
}
