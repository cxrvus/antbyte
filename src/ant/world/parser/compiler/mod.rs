mod assembler;
mod call;
mod func_comp;
mod settings_comp;
mod statement;

use std::{fmt::Display, fs, path::PathBuf};

use super::Parser;

use crate::{
	ant::{
		Behavior,
		compiler::func_comp::compile_funcs,
		world::{
			WorldProperties,
			parser::{AntFunc, ParamValue, Signature, token::Token},
		},
	},
	truth_table::TruthTable,
};

use anyhow::{Context, Result, bail};

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

pub fn compile_world_file(path: &PathBuf, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let code = read_file(path)?;
	compile_world(&code, log_cfg)
		.with_context(|| format!("compiler error in file '{}'", path.to_string_lossy()))
}

fn read_file(path: &PathBuf) -> Result<String> {
	let extension = path.extension().unwrap_or_default().to_string_lossy();

	if extension != "ant" {
		bail!("ant files need to have a '.ant' extension");
	}

	let file_name = path.file_stem().unwrap_or_default().to_string_lossy();

	if !validate_file_name(&file_name) {
		bail!("ant file names need to be in snake_case")
	}

	fs::read_to_string(path)
		.with_context(|| format!("error reading file '{}'", path.to_string_lossy()))
}

#[rustfmt::skip]
fn validate_file_name(file_name: &str) -> bool {
	if let Ok(tokens) = Token::tokenize(file_name)
		&& let Some(Token::Ident(ident)) = tokens.first()
		&& dbg!(ident) == dbg!(file_name)
	{ true }
	else { false }
}

pub fn compile_world(code: &str, log_cfg: &LogConfig) -> Result<WorldProperties> {
	if log_cfg.all {
		println!("\n\n================\n\n");
		println!("{code}");
	}

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
			bail!(
				"tried to assign ID #{target_id} to '{target_name}', but it's already assigned to '{}'",
				behavior.name
			);
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

	if behaviors[1].is_none() {
		behaviors[1] = Some(Default::default());
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
