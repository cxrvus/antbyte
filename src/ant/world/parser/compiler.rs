use std::collections::HashMap;

use super::{Assignment, CircuitType, Expression, ParsedCircuit, Parser, Statement, Token};

use crate::{
	ant::{
		Archetype,
		peripherals::{Input, Output, PeripheralSet},
		world::WorldConfig,
	},
	circuit::Circuit,
};

use anyhow::{Result, anyhow};

struct Graph(Vec<GraphLayer>);
struct GraphLayer(Vec<Node>);
struct Node {
	sign: bool,
	wires: Vec<u32>,
}

pub fn compile(code: String) -> Result<WorldConfig> {
	let parsed_world = Parser::new(code).parse_world()?;

	let mut config = WorldConfig::default();
	let mut parsed_circuits: Vec<(String, ParsedCircuit)> = vec![];

	for statement in parsed_world.statements {
		match statement {
			Statement::Set(key, value) => set_setting(&mut config, key, value)?,
			Statement::Declare(name, circuit) => {
				parsed_circuits.push((name, circuit));
			}
		}
	}

	let resolved_circuits = resolve_circuits(parsed_circuits)?;

	// create Archetypes
	for resolved_circuit in resolved_circuits {
		let circuit = Circuit::new(0, vec![]); // TODO;
		let (_, resolved_circuit) = resolved_circuit;

		if let CircuitType::Ant(ant_type) = resolved_circuit.circuit_type {
			let used_inputs = resolved_circuit
				.used_inputs
				.into_iter()
				.map(Input::from_ident)
				.collect::<Result<Vec<_>>>()?;

			let input_spec = PeripheralSet::from_used(used_inputs, true)?;

			let used_outputs = resolved_circuit
				.used_outputs
				.into_iter()
				.map(Output::from_ident)
				.collect::<Result<Vec<_>>>()?;

			let output_spec = PeripheralSet::from_used(used_outputs, true)?;

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

fn resolve_circuits(
	circuits: Vec<(String, ParsedCircuit)>,
) -> Result<HashMap<String, ParsedCircuit>> {
	let mut resolved_circuits: HashMap<String, ParsedCircuit> = HashMap::new();

	for (name, circuit) in circuits {
		let mut resolved_assignments: Vec<Assignment> = vec![];

		for Assignment {
			lhs: assignees,
			rhs: expression,
		} in circuit.assignments.iter()
		{
			let mut call_stack: Vec<String> = vec![];
			let mut exp_stack: Vec<&Expression> = vec![expression];

			while let Some(exp) = exp_stack.pop() {
				if let Some(parameters) = &exp.parameters {
					parameters
						.iter()
						.rev()
						.for_each(|parameter| exp_stack.push(parameter));
				} else {
					// TODO
					println!("{exp:?}");
				}
			}
		}

		let circuit = ParsedCircuit {
			assignments: resolved_assignments,
			..circuit
		};

		if resolved_circuits.insert(name.clone(), circuit).is_some() {
			return Err(anyhow!("circuit name '{name}' used more than once"));
		}
	}

	Ok(resolved_circuits)
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
