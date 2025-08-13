mod assignment;
mod circuit_comp;
mod settings_comp;

use std::collections::HashMap;

use super::{CircuitType, ParsedCircuit, Parser, Statement};

use crate::{
	ant::{
		Archetype,
		compiler::{circuit_comp::flatten_circuits, settings_comp::set_setting},
		peripherals::{Input, Output, PeripheralSet},
		world::WorldConfig,
	},
	circuit::Circuit,
};

use anyhow::{Ok, Result};

struct Graph(Vec<GraphLayer>);
struct GraphLayer(Vec<Node>);

#[derive(Default)]
struct Normalizer(HashMap<String, FlatCircuit>);

#[derive(Debug)]
struct FlatCircuit {
	original: ParsedCircuit,
	nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
struct Node {
	ident: String,
	sign: bool,
	wires: Vec<Wire>,
}

/// like `Assignment`, but flattened, using `Wire`s instead of recursive `Expression`s
#[derive(Debug, Clone)]
struct FlatAssignment {
	call: String,
	lhs: Vec<String>,
	sign: bool,
	wires: Vec<Wire>,
}

#[derive(Debug, Clone)]
struct Wire {
	sign: bool,
	target: String,
}

impl From<FlatAssignment> for Node {
	fn from(flat_exp: FlatAssignment) -> Self {
		#[rustfmt::skip]
		let FlatAssignment { lhs, sign, wires, ..  } = flat_exp;

		assert_eq!(
			lhs.len(),
			1,
			"FlatAssignment must have exactly one left-hand-side value\n({lhs:?})",
		);

		Self {
			sign,
			ident: lhs[0].clone(),
			wires: wires.clone(),
		}
	}
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

	// create Archetypes
	for flat_circuit in flatten_circuits(parsed_circuits)? {
		let circuit = flat_circuit.1.original;

		if let CircuitType::Ant(ant_type) = circuit.circuit_type {
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
