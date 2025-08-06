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

			// TODO: properly convert to circuit
			let circuit = Circuit::new(0, vec![]);

			// TODO: fill peripherals with sorted inputs from specs, instead of unsorted user-given peripherals

			let archetype = Archetype {
				ant_type,
				circuit,
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
	if let Some(dupe_ident) = circuit
		.used_inputs
		.iter()
		.find(|input| circuit.used_outputs.iter().any(|output| output == *input))
	{
		Err(anyhow!(
			"identifier '{dupe_ident}' used as both input and output"
		))
	} else {
		Ok(())
	}
}

#[derive(Debug)]
struct FlattenedCircuit {
	original: ParsedCircuit,
	assignments: Vec<FlatAssignment>,
}

#[derive(Debug, Clone)]
struct FlatExpression {
	call: String,
	lhs: String,
	sign: bool,
	wires: Vec<Wire>,
}

#[derive(Debug)]
struct FlatAssignment {
	lhs: String,
	sign: bool,
	wires: Vec<Wire>,
}

#[derive(Debug, Clone)]
struct Wire {
	sign: bool,
	target: String,
}

fn flatten_circuit(
	circuit: ParsedCircuit,
	flattened_circuits: &HashMap<String, FlattenedCircuit>,
) -> Result<FlattenedCircuit> {
	let ParsedCircuit {
		name: circuit_name,
		circuit_type,
		used_inputs: inputs,
		used_outputs: outputs,
		assignments,
	} = &circuit;

	let mut flat_assignments: Vec<FlatAssignment> = vec![];

	for (assignment_index, assignment) in assignments.iter().enumerate() {
		let assignment_prefix = format!("_as{assignment_index:02}");

		let mut flat_exps = flatten_expression(&assignment.rhs, &mut 0);

		for flat_exp in flat_exps.iter_mut() {
			for target in flat_exp.wires.iter_mut().map(|wire| &mut wire.target) {
				// verifying identifiers in the flat exp

				let is_in_input = inputs.contains(target);

				if !is_in_input {
					*target = assignment_prefix.clone() + target;
				}

				let is_declared = is_in_input || flat_assignments.iter().any(|x| x.lhs == *target);

				if !is_declared {
					let error = if flattened_circuits.contains_key(target) {
						anyhow!("'{target}' is a circuit, not an input")
					} else if outputs.contains(target) {
						anyhow!("'{target}' is an output, not an input")
					} else {
						anyhow!("unknown identifier: '{target}'")
					};

					return Err(error);
				}
			}

			// TODO: resolve function calls

			// TODO: flatten assignment LHSs

			// dbg!(&flat_exp);

			#[rustfmt::skip]
			let FlatExpression { lhs, sign, wires, ..  } = flat_exp.clone();
			let lhs = assignment_prefix.clone() + &lhs;
			let flat_assignment = FlatAssignment { lhs, sign, wires };

			flat_assignments.push(flat_assignment);
		}

		// TODO: remove the [0]
		if let Some(dupe_assignment) = flat_assignments.iter().find(|x| x.lhs == assignment.lhs[0])
		{
			return Err(anyhow!(
				"identifier '{}' can not be assigned to more than once",
				dupe_assignment.lhs
			));
		} else {
			flat_assignments.last_mut().unwrap().lhs = assignment.lhs[0].clone();
		}

		println!("\n\n\n") //TODO: remove (dbg)
	}

	dbg!(&flat_assignments);

	Ok(FlattenedCircuit {
		assignments: flat_assignments,
		original: circuit,
	})
}

#[inline]
fn format_index(index: u32) -> String {
	format!("_exp{index:02}")
}

fn flatten_expression(exp: &Expression, index: &mut u32) -> Vec<FlatExpression> {
	let mut flat_exps = vec![];

	let (call, wires) = if let Some(parameters) = &exp.parameters {
		let mut wires = vec![];

		for sub_exp in parameters {
			if sub_exp.parameters.is_some() {
				flat_exps.extend(flatten_expression(sub_exp, index));

				wires.push(Wire {
					sign: sub_exp.sign,
					target: format_index(*index - 1),
				});
			} else {
				wires.push(Wire {
					sign: sub_exp.sign,
					target: sub_exp.ident.clone(),
				});
			}
		}

		(exp.ident.clone(), wires)
	} else {
		let wires = vec![Wire {
			sign: exp.sign,
			target: exp.ident.clone(),
		}];

		("or".to_string(), wires)
	};

	flat_exps.push(FlatExpression {
		call,
		lhs: format_index(*index),
		sign: exp.sign,
		wires,
	});

	*index += 1;
	flat_exps
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
