mod func_comp;
mod settings_comp;
mod statement;

use std::collections::HashMap;

use super::Parser;

use crate::ant::world::{WorldConfig, parser::Signature};

use anyhow::{Ok, Result};

#[derive(Default)]
pub struct Compiler {
	world_config: WorldConfig,
	comp_funcs: HashMap<String, CompFunc>,
}

#[derive(Debug)]
struct CompFunc {
	signature: Signature,
	comp_statements: Vec<CompStatement>,
}

/// like `Statement`, but flattened, using `ParamValue`s instead of recursive `Expression`s
#[derive(Debug, Clone)]
struct FlatStatement {
	func: String,
	assignees: Vec<String>,
	sign: bool,
	params: Vec<ParamValue>,
}

/// like `FlatStatement`, but with exactly one assignee
/// and without `func`: all funcs resolved to be `OR`
#[derive(Debug, Clone)]
struct CompStatement {
	assignee: String,
	sign: bool,
	params: Vec<ParamValue>,
}

#[derive(Debug, Clone)]
struct ParamValue {
	sign: bool,
	target: String,
}

impl From<FlatStatement> for CompStatement {
	fn from(flat_statement: FlatStatement) -> Self {
		#[rustfmt::skip]
		let FlatStatement { assignees, sign, params, func } = flat_statement;

		assert_eq!(
			func, "or",
			"FlatStatement func must be 'or' \nfound '{func}'"
		);

		assert_eq!(
			assignees.len(),
			1,
			"FlatStatement must have exactly one left-hand-side value\nfound {assignees:?})",
		);

		Self {
			sign,
			assignee: assignees[0].clone(),
			params: params.clone(),
		}
	}
}

impl Compiler {
	pub fn compile(code: String) -> Result<WorldConfig> {
		let parsed_world = Parser::new(code).parse_world()?;

		let mut compiler = Self::default();

		for (key, value) in parsed_world.settings {
			compiler.set_setting(key, value)?;
		}

		compiler.compile_funcs(parsed_world.funcs)?;

		todo!("CONTINUE");

		// dbg!(&config);

		Ok(compiler.world_config)
	}
}
