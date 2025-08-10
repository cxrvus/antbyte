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

	let mut flat_circuits: HashMap<String, FlatCircuit> = HashMap::new();

	for circuit in parsed_circuits.into_iter() {
		validate_circuit_io(&circuit)?;

		let circuit_name = circuit.name.clone();
		let flat_circuit = flatten_circuit(circuit, &flat_circuits)?;

		if flat_circuits
			.insert(circuit_name.clone(), flat_circuit)
			.is_some()
		{
			return Err(anyhow!("circuit name '{circuit_name}' used more than once"));
		}
	}

	// create Archetypes
	for flat_circuit in flat_circuits {
		let circuit = flat_circuit.1.original;

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
struct FlatCircuit {
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

#[derive(Debug, Clone)]
struct FlatAssignment {
	lhs: String,
	sign: bool,
	wires: Vec<Wire>,
}

impl From<FlatExpression> for FlatAssignment {
	fn from(flat_exp: FlatExpression) -> Self {
		#[rustfmt::skip]
		let FlatExpression { lhs, sign, wires, ..  } = flat_exp;
		Self { lhs, sign, wires }
	}
}

#[derive(Debug, Clone)]
struct Wire {
	sign: bool,
	target: String,
}

fn flatten_circuit(
	circuit: ParsedCircuit,
	flat_circuits: &HashMap<String, FlatCircuit>,
) -> Result<FlatCircuit> {
	let ParsedCircuit {
		name: circuit_name,
		circuit_type,
		used_inputs: inputs,
		used_outputs: outputs,
		assignments,
	} = &circuit;

	let mut exp_index = 0;
	let mut func_index = 0;
	let mut flat_assignments: Vec<FlatAssignment> = vec![];

	for assignment in assignments.iter() {
		let mut flat_exps = flatten_expression(&assignment.rhs, &mut exp_index);

		exp_index += 1;

		for flat_exp in flat_exps.iter_mut() {
			// TODO: encapsulate this and the following statements in FlatExpression methods
			// verifying identifiers in the flat exp
			for target in flat_exp.wires.iter_mut().map(|wire| &mut wire.target) {
				let is_in_input = inputs.contains(target);
				let is_declared = is_in_input || flat_assignments.iter().any(|x| x.lhs == *target);

				if !is_declared {
					let error = if flat_circuits.contains_key(target) {
						anyhow!("'{target}' is a circuit, not an input")
					} else if outputs.contains(target) {
						anyhow!("'{target}' is an output, not an input")
					} else {
						anyhow!("unknown identifier: '{target}'")
					};

					return Err(error);
				}
			}

			// resolve calls
			match flat_exp.call.as_str() {
				"or" => {
					// already resolved
					flat_assignments.push(flat_exp.clone().into());
				}
				"and" => {
					// TODO: resolve beforehand using separate iteration

					// transform AND into OR [DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws)

					flat_exp
						.wires
						.iter_mut()
						.for_each(|wire| wire.sign = !wire.sign);
					flat_exp.sign = !flat_exp.sign;

					flat_assignments.push(flat_exp.clone().into());
				}
				call => {
					let func = flat_circuits
						.get(call)
						.ok_or(anyhow!("unknown function: '{call}'"))?;

					if let CircuitType::Ant(ant_type) = &func.original.circuit_type {
						return Err(anyhow!(
							"circuit '{call}' is a {ant_type:?}, not a function"
						));
					}

					// TODO verify input count
					// TODO verify output count

					let var_prefix = format!("_fn_{call}{func_index:02}");

					for mut func_assignment in func.assignments.clone() {
						if let Some(output_index) = func
							.original
							.used_outputs
							.iter()
							.position(|output| *output == func_assignment.lhs)
						{
							func_assignment.lhs = assignment.lhs[output_index].clone();
						} else {
							func_assignment.lhs = var_prefix.clone() + &func_assignment.lhs;
						}

						for func_assignment_wire in func_assignment.wires.iter_mut() {
							if let Some(func_param_index) = func
								.original
								.used_inputs
								.iter()
								.position(|input| *input == func_assignment_wire.target)
							{
								let input_wire = &flat_exp.wires[func_param_index];
								func_assignment_wire.target = input_wire.target.clone();
								func_assignment_wire.sign ^= input_wire.sign;
							} else {
								// TODO: create prefix_var() function
								func_assignment_wire.target =
									var_prefix.clone() + &func_assignment_wire.target;
							}
						}

						flat_assignments.push(func_assignment);
					}

					func_index += 1;
				}
			}

			// TODO: flatten assignment LHSs

			// dbg!(&flat_exp);
		}

		// TODO: FIX - this error gets thrown mistakenly,
		// due to assignment insertion during function expansion
		// => move this into the OR bock
		// (add function that always checks if distinct before pushing?)

		// TODO: remove the [0] by iterating thru LHSs
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

	Ok(FlatCircuit {
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
