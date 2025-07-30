use std::process::Output;

use crate::{
	ant::{
		archetype::Archetype,
		peripherals::{Input, InputType, Peripheral, PeripheralSet},
	},
	compiler::{
		parser::{ParsedCircuit, Parser, Setting, Statement},
		token::Token,
	},
	world::WorldConfig,
};
use anyhow::{Error, Result, anyhow};

pub mod parser;
pub mod token;

const PERIPH_PTN: &str = r"^([A_Z]{1,4})([0-9a-f])?$";

pub fn compile(code: String) -> Result<WorldConfig> {
	let parsed_world = Parser::parse(code)?;

	let mut config = WorldConfig::default();
	let mut parsed_circuits: Vec<ParsedCircuit> = vec![];

	for statement in parsed_world.statements {
		match statement {
			Statement::Set(setting) => set_setting(&mut config, setting)?,
			Statement::Declare(circuit) => parsed_circuits.push(circuit),
		}
	}

	for parsed_circuit in parsed_circuits {
		match parsed_circuit.circuit_type {
			parser::CircuitType::Ant(ant_type) => {
				let mut inputs: Vec<Input>;
				let mut outputs: Vec<Output>;

				for parsed_input in parsed_circuit.inputs {}

				for parsed_output in parsed_circuit.outputs {}

				let archetype = Archetype {
					ant_type,
					circuit: todo!(),
					inputs: PeripheralSet::inputs(inputs)?,
					outputs: todo!(),
				};

				config.archetypes.push(archetype);
			}

			parser::CircuitType::Sub => {
				todo!()
			}
		};
	}

	dbg!(&config);

	Ok(config)
}

fn set_setting(config: &mut WorldConfig, setting: Setting) -> Result<()> {
	let Setting { key, value } = setting;

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
