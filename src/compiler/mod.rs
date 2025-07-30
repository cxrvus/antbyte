use crate::{
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
	let mut circuits = Vec::<ParsedCircuit>::new();

	for statement in parsed_world.statements {
		match statement {
			Statement::Set(setting) => set_setting(&mut config, setting)?,
			Statement::Declare(circuit) => circuits.push(circuit),
		}
	}

	for circuit in circuits {
		todo!()
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
