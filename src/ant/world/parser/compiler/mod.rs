mod func_comp;
mod settings_comp;
mod statement;

use std::collections::HashMap;

use super::{Func, FuncType, GlobalStatement, Parser};

use crate::ant::{compiler::settings_comp::set_setting, world::WorldConfig};

use anyhow::{Ok, Result};

#[derive(Default)]
struct Compiler(HashMap<String, NormFunc>);

#[derive(Debug)]
struct NormFunc {
	original: Func,
	norm_statements: Vec<NormStatement>,
}

/// like `Statement`, but flattened, using `ParamValue`s instead of recursive `Expression`s
#[derive(Debug, Clone)]
struct FlatStatement {
	call: String,
	assignees: Vec<String>,
	sign: bool,
	params: Vec<ParamValue>,
}

/// like `FlatStatement`, but with exactly one assignee
/// and without `call`: all calls normalized to `OR`
#[derive(Debug, Clone)]
struct NormStatement {
	assignee: String,
	sign: bool,
	params: Vec<ParamValue>,
}

#[derive(Debug, Clone)]
struct ParamValue {
	sign: bool,
	target: String,
}

impl From<FlatStatement> for NormStatement {
	fn from(flat_statement: FlatStatement) -> Self {
		#[rustfmt::skip]
		let FlatStatement { assignees, sign, params, call  } = flat_statement;

		assert_eq!(
			call, "or",
			"FlatStatement call must be 'or' \nfound '{call}'"
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

pub fn compile(code: String) -> Result<WorldConfig> {
	let global_statements = Parser::new(code).parse_world()?;

	let mut config = WorldConfig::default();
	let mut funcs: Vec<Func> = vec![];

	for global_statement in global_statements {
		match global_statement {
			GlobalStatement::Set(key, value) => set_setting(&mut config, key, value)?,
			GlobalStatement::Declare(func) => {
				funcs.push(func);
			}
		}
	}

	// create Behaviors
	for (name, norm_func) in Compiler::normalize_funcs(funcs)? {
		let NormFunc {
			original: func,
			norm_statements,
		} = norm_func;

		if let FuncType::Ant(ant_type) = func.func_type.clone() {
			todo!("continue");
		};
	}

	// dbg!(&config);

	Ok(config)
}
