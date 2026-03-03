mod assembler;
mod call;
mod func_comp;
pub mod settings_comp;
mod statement;
mod stdlib;
mod test_std;

use std::{
	collections::{BTreeMap, HashSet},
	fmt::Display,
	mem::take,
	path::{Path, PathBuf},
};

use crate::{
	ant::{
		Behavior,
		compiler::{func_comp::compile_funcs, stdlib::STDLIB},
		world::WorldProperties,
	},
	files::read_file,
	parser::{
		AntFunc, Expression, Func, ParamValue, Parser, Signature, SignatureSpec, func_parser::MAIN,
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

	if !parsed_world.no_std {
		let std_funcs = Parser::new(STDLIB)?.parse_world().unwrap().funcs;
		parsed_funcs.extend(std_funcs);
	}

	let mut imported = HashSet::new();

	eprintln!("Linking...");

	for import in &parsed_world.imports {
		let path = if let Some(source_path) = source_path {
			let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
			base_dir.join(format!("{import}.ant"))
		} else {
			bail!("cannot import other files in path-less compilations");
		};

		import_funcs(&path, &mut parsed_funcs, &mut imported)?;
	}

	// add source file's functions after imports so imported functions are available
	parsed_funcs.extend(take(&mut parsed_world.funcs));

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

fn import_funcs(
	path: &PathBuf,
	parsed_funcs: &mut Vec<Func>,
	imported: &mut HashSet<PathBuf>,
) -> Result<()> {
	import_funcs_recursive(path, parsed_funcs, imported, &mut HashSet::new())
		.with_context(|| format!("in file '{}'!", path.to_string_lossy()))
}

fn import_funcs_recursive(
	path: &PathBuf,
	parsed_funcs: &mut Vec<Func>,
	imported: &mut HashSet<PathBuf>,
	visiting: &mut HashSet<PathBuf>,
) -> Result<()> {
	if visiting.contains(path) {
		bail!("circular import detected: '{}'", path.to_string_lossy());
	}

	if imported.contains(path) {
		return Ok(());
	}

	visiting.insert(path.clone());
	imported.insert(path.clone());

	let code = read_file(path)?;
	let parsed_world = Parser::new(&code)?.parse_world()?;

	let base_dir = path.parent().unwrap_or_else(|| Path::new("."));

	for import in &parsed_world.imports {
		let import_path = base_dir.join(format!("{import}.ant"));
		import_funcs_recursive(&import_path, parsed_funcs, imported, visiting)?
	}

	let mut new_parsed_funcs = parsed_world.funcs;
	sanitize_main(&mut new_parsed_funcs, path);
	parsed_funcs.extend(new_parsed_funcs);

	visiting.remove(path);
	Ok(())
}

fn sanitize_main(parsed_funcs: &mut [Func], path: &Path) {
	for parsed_func in parsed_funcs.iter_mut() {
		let file_name = path.file_stem().unwrap().to_string_lossy().to_string();

		for stm in parsed_func.statements.iter_mut() {
			sanitize_exp(&mut stm.expression, &file_name);
		}

		if parsed_func.signature.name == MAIN {
			parsed_func.signature.name = file_name;
		}
	}

	fn sanitize_exp(exp: &mut Expression, file_name: &str) {
		if let Some(params) = &mut exp.params {
			if exp.ident == MAIN {
				exp.ident = file_name.to_owned();
			}

			for sub_exp in params.iter_mut() {
				sanitize_exp(sub_exp, file_name);
			}
		}
	}
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
