mod assembler;
mod call;
mod func_comp;
mod settings_comp;
mod statement;

use std::fmt::Display;

use super::Parser;

use crate::{
	ant::{
		Behavior,
		compiler::func_comp::compile_funcs,
		world::{
			WorldProperties,
			parser::{AntFunc, ParamValue, Signature},
		},
	},
	truth_table::TruthTable,
};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
struct CompFunc {
	signature: Signature,
	comp_statements: Vec<CompStatement>,
}

impl Display for CompFunc {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{} !{{", self.signature)?;

		for comp_statement in &self.comp_statements {
			writeln!(f, "\t{comp_statement}")?
		}

		writeln!(f, "}}")
	}
}

/// like `Statement`, but flattened, using `ParamValue`s instead of recursive `Expression`s
#[derive(Debug, Clone)]
struct FuncCall {
	func: String,
	assignees: Vec<ParamValue>,
	params: Vec<ParamValue>,
}

/// like `FuncCall`, but without func and with exactly one assignee
/// and without `func`: all funcs resolved to be `OR`
#[derive(Debug, Clone)]
struct CompStatement {
	assignee: ParamValue,
	params: Vec<ParamValue>,
}

pub struct LogConfig {
	pub all: bool,
}

pub fn compile_world(code: &str, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let parsed_world = Parser::new(code)?.parse_world()?;

	let mut properties = WorldProperties::default();

	for (key, value) in parsed_world.settings {
		properties.config.set_setting(key, value)?;
	}

	let comp_funcs = compile_funcs(parsed_world.funcs)?;

	let mut behaviors: [Option<Behavior>; 0x100] = [const { None }; 0x100];

	for AntFunc {
		target_name,
		target_id,
	} in parsed_world.ants
	{
		if let Some(behavior) = &behaviors[target_id as usize] {
			return Err(anyhow!(
				"tried to assign ID #{target_id} to '{target_name}', but it's already assigned to '{}'",
				behavior.name
			));
		} else {
			// a call with no params or assignees to emulate the conditions for a valid ant Func
			let func_call = FuncCall {
				func: target_name,
				assignees: vec![],
				params: vec![],
			};

			let target_func = func_call.get_overload(&comp_funcs).unwrap();

			let behavior = target_func.assemble(log_cfg).map(Some)?;
			behaviors[target_id as usize] = behavior;
		}
	}

	properties.behaviors = behaviors;

	Ok(properties)
}

pub fn compile_main(code: String) -> TruthTable {
	let parsed_world = Parser::new(&code).unwrap().parse_world().unwrap();

	assert_eq!(parsed_world.settings.len(), 0);
	assert_eq!(parsed_world.ants.len(), 0);

	let log_cfg = LogConfig { all: true };

	compile_funcs(parsed_world.funcs)
		.unwrap()
		.iter()
		.find(|x| x.signature.name == "main")
		.expect("'main' function required for compile_main")
		.assemble(&log_cfg)
		.unwrap()
		.logic
}
