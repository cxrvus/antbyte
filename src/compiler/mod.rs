use crate::{
	ant::{
		archetype::Archetype,
		peripherals::{Input, InputType, Output, Peripheral, PeripheralSet},
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
				let inputs = parsed_circuit
					.inputs
					.into_iter()
					.map(Input::from_ident)
					.collect::<Result<_>>()?;

				let outputs = parsed_circuit
					.outputs
					.into_iter()
					.map(Output::from_ident)
					.collect::<Result<_>>()?;

				dbg!(&inputs, &outputs);

				let archetype = Archetype {
					ant_type,
					circuit: todo!(),
					outputs: PeripheralSet::outputs(outputs)?,
					inputs: PeripheralSet::inputs(inputs)?,
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
