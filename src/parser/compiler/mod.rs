mod assembler;
mod call;
mod func_comp;
pub mod linker;
pub mod settings_comp;
mod statement;
mod stdlib;
mod test_std;

use std::{
	collections::BTreeMap,
	fmt::Display,
	mem::{self, take},
	path::PathBuf,
};

use crate::{
	ant::behavior::Behavior,
	parser::{
		AntFunc, ParamValue, Parser, Signature, SignatureSpec,
		compiler::{func_comp::compile_funcs, stdlib::STDLIB},
	},
	truth_table::TruthTable,
	world::WorldProperties,
};

use anyhow::{Result, bail};

#[derive(Debug, Clone)]
struct CompFunc {
	signature: Signature,
	comp_statements: Vec<CompStatement>,
}

impl Display for CompFunc {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{} ~{{", self.signature)?;

		for comp_statement in &self.comp_statements {
			writeln!(f, "\t{comp_statement}")?;

			if comp_statement.params.len() > 1 {
				writeln!(f)?;
			}
		}

		writeln!(f, "}}")
	}
}

/// like `Statement`, but flattened, using `ParamValue`s instead of recursive `Expression`s
#[derive(Debug, Clone)]
pub struct FuncCall {
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

#[derive(Default)]
pub struct LogConfig {
	pub all: bool,
}

pub fn compile_world(
	code: &str,
	log_cfg: &LogConfig,
	source_path: Option<&PathBuf>,
) -> Result<WorldProperties> {
	if log_cfg.all {
		eprintln!("\n\n========LOG========\n\n");
		eprintln!("{code}");
	}

	eprintln!("Parsing...");

	let mut parsed_world = Parser::new(code)?.parse_world()?;

	let mut parsed_funcs = vec![];
	let mut imported_settings = vec![];

	if !parsed_world.no_std {
		let std_funcs = Parser::new(STDLIB)?.parse_world().unwrap().funcs;
		parsed_funcs.extend(std_funcs);
	}

	eprintln!("Linking...");

	linker::link(
		source_path,
		&parsed_world.imports,
		&mut parsed_funcs,
		&mut imported_settings,
	)?;

	// add source file's functions after imports so imported functions are available
	parsed_funcs.extend(take(&mut parsed_world.funcs));

	// apply imported settings before local settings
	imported_settings.extend(mem::take(&mut parsed_world.settings));
	parsed_world.settings = imported_settings;

	let mut properties = WorldProperties::default();

	for (key, value) in parsed_world.settings {
		properties.config.set_setting(key, value)?;
	}

	eprintln!("Compiling...");

	let comp_funcs = compile_funcs(parsed_funcs, log_cfg)?;

	let mut behaviors: BTreeMap<u8, Behavior> = BTreeMap::new();

	for AntFunc {
		target_name,
		target_id,
	} in parsed_world.ants
	{
		eprintln!("Assembling ant '{target_name}' @ {target_id}...");

		if let Some(behavior) = behaviors.get(&target_id) {
			bail!(
				"tried to assign ID #{target_id} to '{target_name}', but it's already assigned to '{}'",
				behavior.name
			);
		} else {
			// a signature spec with no params or assignees to emulate the conditions for a valid ant Func
			let signature = SignatureSpec {
				name: &target_name,
				assignee_count: 0,
				param_count: 0,
			};

			let target_func = signature.get_overload(&comp_funcs).unwrap();
			let behavior = target_func.assemble(log_cfg)?;
			behaviors.insert(target_id, behavior);
		}
	}

	properties.behaviors = behaviors;

	Ok(properties)
}

pub fn compile_world_simple(code: &str) -> Result<WorldProperties> {
	compile_world(code, &Default::default(), None)
}

pub fn compile_func(code: &str, signature: SignatureSpec) -> TruthTable {
	let parsed_world = Parser::new(code).unwrap().parse_world().unwrap();
	let comp_funcs = compile_funcs(parsed_world.funcs, &LogConfig::default()).unwrap();
	let func = signature.get_overload(&comp_funcs).unwrap();
	let log_cfg = LogConfig { all: true };

	func.assemble(&log_cfg).unwrap().logic
}
