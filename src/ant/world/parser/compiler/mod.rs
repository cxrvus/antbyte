mod func_comp;
mod settings_comp;
mod statement;

use super::Parser;

use crate::ant::{
	compiler::func_comp::compile_funcs,
	world::{
		World,
		parser::{ParamValue, Signature},
	},
};

use anyhow::{Ok, Result};

#[derive(Debug)]
struct CompFunc {
	signature: Signature,
	comp_statements: Vec<CompStatement>,
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

pub fn compile(code: String) -> Result<World> {
	let parsed_world = Parser::new(code).parse_world()?;

	let mut world = World::default();

	for (key, value) in parsed_world.settings {
		world.set_setting(key, value)?;
	}

	let comp_funcs = compile_funcs(parsed_world.funcs)?;

	todo!("CONTINUE");

	Ok(world)
}
