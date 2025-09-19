mod assembler;
mod call;
mod func_comp;
mod settings_comp;
mod statement;
mod stdlib;
mod test_std;

use std::{
	collections::HashSet,
	fmt::Display,
	fs,
	mem::take,
	path::{Path, PathBuf},
};

use super::Parser;

use crate::{
	ant::{
		Behavior,
		compiler::func_comp::compile_funcs,
		world::{
			WorldProperties,
			parser::{AntFunc, Func, ParamValue, Signature, SignatureSpec, token::Token},
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

pub fn compile_world_file(path: &PathBuf, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let code = read_file(path)?;
	compile_world(&code, log_cfg, Some(path))
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
		&& ident == file_name
	{ true }
	else { false }
}

pub fn compile_world(
	code: &str,
	log_cfg: &LogConfig,
	source_path: Option<&PathBuf>,
) -> Result<WorldProperties> {
	if log_cfg.all {
		println!("\n\n========LOG========\n\n");
		println!("{code}");
	}

	let mut parsed_world = Parser::new(code)?.parse_world()?;

	let mut parsed_funcs = vec![];
	let mut visited = HashSet::new();

	for import in &parsed_world.imports {
		let path = if let Some(source_path) = source_path {
			let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
			base_dir.join(format!("{import}.ant"))
		} else {
			bail!("cannot import other files in path-less compilations");
		};

		import_funcs(&path, &mut parsed_funcs, &mut visited)?;
	}

	// add source file's functions after imports so imported functions are available
	parsed_funcs.extend(take(&mut parsed_world.funcs));

	let mut properties = WorldProperties::default();

	for (key, value) in parsed_world.settings {
		properties.config.set_setting(key, value)?;
	}

	let comp_funcs = compile_funcs(parsed_funcs)?;

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
			// a signature spec with no params or assignees to emulate the conditions for a valid ant Func
			let signature = SignatureSpec {
				name: &target_name,
				assignee_count: 0,
				param_count: 0,
			};

			let target_func = signature.get_overload(&comp_funcs).unwrap();
			let behavior = target_func.assemble(log_cfg).map(Some)?;
			behaviors[target_id as usize] = behavior;
		}
	}

	properties.behaviors = behaviors;

	Ok(properties)
}

fn import_funcs(
	path: &PathBuf,
	parsed_funcs: &mut Vec<Func>,
	visited: &mut HashSet<PathBuf>,
) -> Result<()> {
	import_funcs_recursive(path, parsed_funcs, visited)
		.with_context(|| format!("in file '{}'", path.to_string_lossy()))
}

fn import_funcs_recursive(
	path: &PathBuf,
	parsed_funcs: &mut Vec<Func>,
	visited: &mut HashSet<PathBuf>,
) -> Result<()> {
	if visited.contains(path) {
		bail!("circular import detected: '{}'", path.to_string_lossy());
	}

	visited.insert(path.clone());

	let code = read_file(path)?;
	let parsed_world = Parser::new(&code)?.parse_world()?;

	let base_dir = path.parent().unwrap_or_else(|| Path::new("."));

	for import in &parsed_world.imports {
		let import_path = base_dir.join(format!("{import}.ant"));
		import_funcs(&import_path, parsed_funcs, visited)?
	}

	parsed_funcs.extend(parsed_world.funcs);

	visited.remove(path);
	Ok(())
}

pub fn compile_world_simple(code: &str) -> Result<WorldProperties> {
	compile_world(code, &Default::default(), None)
}

pub fn compile_func(code: &str, signature: SignatureSpec) -> TruthTable {
	let parsed_world = Parser::new(code).unwrap().parse_world().unwrap();
	let comp_funcs = compile_funcs(parsed_world.funcs).unwrap();
	let func = signature.get_overload(&comp_funcs).unwrap();
	let log_cfg = LogConfig { all: true };

	func.assemble(&log_cfg).unwrap().logic
}
