mod assignment;
mod circuit_comp;
mod graph;
mod settings_comp;

use std::collections::HashMap;

use super::{CircuitType, GlobalStatement, ParsedCircuit, Parser};

use crate::ant::{
	Archetype,
	compiler::{circuit_comp::flatten_circuits, graph::create_graph, settings_comp::set_setting},
	world::WorldConfig,
};

use anyhow::{Ok, Result};

#[derive(Debug)]
struct Graph(Vec<GraphLayer>);

#[derive(Debug)]
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
	assignees: Vec<String>,
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
		let FlatAssignment { assignees, sign, wires, ..  } = flat_exp;

		assert_eq!(
			assignees.len(),
			1,
			"FlatAssignment must have exactly one left-hand-side value\n({assignees:?})",
		);

		Self {
			sign,
			ident: assignees[0].clone(),
			wires: wires.clone(),
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
		let FlatCircuit {
			original: parsed_circuit,
			nodes,
		} = flat_circuit;

		if let CircuitType::Ant(ant_type) = parsed_circuit.circuit_type.clone() {
			let graph = create_graph(parsed_circuit, &nodes)?;
			let circuit = graph.into();

			let archetype = Archetype {
				ant_type: ant_type.clone(),
				circuit,
				outputs: todo!(),
				inputs: todo!(),
			};

			config.archetypes.push(archetype);
		};
	}

	// dbg!(&config);

	Ok(config)
}
