use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{
	parser::{Expression, Func, Parser},
	world::file_compiler::read_file,
};

const MAIN: &str = "main";

pub fn link(
	source_path: Option<&PathBuf>,
	imports: &Vec<String>,
	parsed_funcs: &mut Vec<Func>,
) -> Result<()> {
	let mut imported = HashSet::new();
	for import in imports {
		let path = if let Some(source_path) = source_path {
			let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
			base_dir.join(format!("{import}.ant"))
		} else {
			bail!("cannot import other files in path-less compilations");
		};
		import_funcs(&path, parsed_funcs, &mut imported)?;
	}

	Ok(())
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
