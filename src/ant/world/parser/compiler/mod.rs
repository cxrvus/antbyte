mod func_comp;
mod settings_comp;
mod statement;

use std::collections::HashMap;

use super::Parser;

use crate::ant::{
	compiler::func_comp::compile_funcs,
	world::{World, parser::Signature},
};

use anyhow::{Ok, Result};

// TODO: turn this into a vector and implement overloading
type CompFuncs = HashMap<String, CompFunc>;

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
