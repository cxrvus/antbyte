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

use crate::{
	ant::{
		Behavior,
		compiler::{func_comp::compile_funcs, stdlib::STDLIB},
		world::{
			WorldProperties,
			parser::{
				AntFunc, Expression, Func, ParamValue, Parser, Signature, SignatureSpec,
				func_parser::MAIN, token::Token,
			},
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

pub fn compile_world_file(path: &PathBuf, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let code = read_file(path)?;
	compile_world(&code, log_cfg, Some(path))
		.with_context(|| format!("compiler error in file '{}'!", path.to_string_lossy()))
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
		.with_context(|| format!("error reading file '{}'!", path.to_string_lossy()))
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

	println!("Parsing...");

	let mut parsed_world = Parser::new(code)?.parse_world()?;

	let mut parsed_funcs = vec![];

	if !parsed_world.no_std {
		let std_funcs = Parser::new(STDLIB)?.parse_world().unwrap().funcs;
		parsed_funcs.extend(std_funcs);
	}

	let mut imported = HashSet::new();

	println!("Linking...");

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

	println!("Compiling...");

	let comp_funcs = compile_funcs(parsed_funcs, log_cfg)?;

	let mut behaviors: [Option<Behavior>; 0x100] = [const { None }; 0x100];

	for AntFunc {
		target_name,
		target_id,
	} in parsed_world.ants
	{
		println!("Assembling ant '{target_name}' @ {target_id}.");

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
