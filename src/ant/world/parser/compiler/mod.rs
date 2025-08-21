mod circuit_comp;
mod settings_comp;
mod statement;

use std::collections::HashMap;

use super::{CircuitType, GlobalStatement, ParsedCircuit, Parser};

use crate::ant::{
	compiler::{circuit_comp::flatten_circuits, settings_comp::set_setting},
	world::WorldConfig,
};

use anyhow::{Ok, Result};

#[derive(Default)]
struct Normalizer(HashMap<String, NormCircuit>);

#[derive(Debug)]
struct NormCircuit {
	original: ParsedCircuit,
	norm_statements: Vec<NormStatement>,
}

/// like `Statement`, but flattened, using `ParamValue`s instead of recursive `Expression`s
#[derive(Debug, Clone)]
struct FlatStatement {
	call: String,
	assignees: Vec<String>,
	sign: bool,
	params: Vec<ParamValue>,
}

/// like `FlatStatement`, but with exactly one assignee
/// and without `call`: all calls normalized to `OR`
#[derive(Debug, Clone)]
struct NormStatement {
	assignee: String,
	sign: bool,
	params: Vec<ParamValue>,
}

#[derive(Debug, Clone)]
struct ParamValue {
	sign: bool,
	target: String,
}

impl From<FlatStatement> for NormStatement {
	fn from(flat_statement: FlatStatement) -> Self {
		#[rustfmt::skip]
		let FlatStatement { assignees, sign, params, ..  } = flat_statement;

		assert_eq!(
			assignees.len(),
			1,
			"FlatStatement must have exactly one left-hand-side value\n({assignees:?})",
		);

		Self {
			sign,
			assignee: assignees[0].clone(),
			params: params.clone(),
		}
	}
}

pub fn compile(code: String) -> Result<WorldConfig> {
	let parsed_world = Parser::new(code).parse_world()?;

	let mut config = WorldConfig::default();
	let mut parsed_circuits: Vec<ParsedCircuit> = vec![];

	for global_statement in parsed_world.statements {
		match global_statement {
			GlobalStatement::Set(key, value) => set_setting(&mut config, key, value)?,
			GlobalStatement::Declare(circuit) => {
				parsed_circuits.push(circuit);
			}
		}
	}

	// create Archetypes
	for (name, flat_circuit) in flatten_circuits(parsed_circuits)? {
		let NormCircuit {
			original: parsed_circuit,
			norm_statements,
		} = flat_circuit;

		if let CircuitType::Ant(ant_type) = parsed_circuit.circuit_type.clone() {
			todo!("continue");
			// TODO: convert to Circuits

			// let archetype = Archetype {
			// 	ant_type: ant_type.clone(),
			// 	circuit,
			// 	outputs: todo!(),
			// 	inputs: todo!(),
			// };

			// config.archetypes.push(archetype);
		};
	}

	// dbg!(&config);

	Ok(config)
}
