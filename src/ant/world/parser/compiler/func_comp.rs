use anyhow::{Result, anyhow};

use super::CompFunc;

use crate::{
	ant::{
		compiler::{CompFuncs, CompStatement},
		world::parser::{Func, Signature},
	},
	util::find_dupe,
};

pub(super) fn compile_funcs(funcs: Vec<(String, Func)>) -> Result<CompFuncs> {
	let mut comp_funcs = CompFuncs::default();

	for (name, func) in funcs.into_iter() {
		func.signature.validate()?;

		println!("{name}:\n");

		let comp_func = func.compile(&comp_funcs)?;

		if comp_funcs.insert(name.clone(), comp_func).is_some() {
			return Err(anyhow!("func name '{name}' used more than once"));
		}
	}

	Ok(comp_funcs)
}

impl Signature {
	fn validate(&self) -> Result<()> {
		if let Some(dupe_ident) = self.in_params.iter().find(|in_param| {
			self.out_params
				.iter()
				.any(|out_param| out_param == *in_param)
		}) {
			Err(anyhow!(
				"identifier '{dupe_ident}' used both as an in- and an out-parameter"
			))
		} else {
			Ok(())
		}
	}
}

impl Func {
	fn compile(&self, comp_funcs: &CompFuncs) -> Result<CompFunc> {
		let mut exp_index = 0;
		let mut func_index = 0;
		let mut comp_statements: Vec<CompStatement> = vec![];

		for statement in self.statements.iter() {
			exp_index += 1;

			let mut flat_statements = statement.expand_expression(&mut exp_index);

			flat_statements
				.iter_mut()
				.for_each(|stm| stm.resolve_and_gate());

			for flat_statement in flat_statements {
				match flat_statement.func.as_str() {
					"or" => {
						// TODO: implement n:1 expansion
						if flat_statement.assignees.len() != 1 {
							return Err(anyhow!(
								"the result of an OR may only be assigned to a single assignee"
							));
						}

						comp_statements.push(flat_statement.into());
					}
					func_name => {
						func_index += 1;
						let expanded =
							flat_statement.expand_call(comp_funcs, func_name, func_index)?;
						comp_statements.extend(expanded);
					}
				}
			}

			println!() //TODO: remove (dbg)
		}

		let all_assignees: Vec<_> = comp_statements.iter().map(|stm| &stm.assignee).collect();

		if let Some(dupe_assignee) = find_dupe(&all_assignees) {
			return Err(anyhow!(
				"identifier '{dupe_assignee}' can not be assigned to more than once"
			));
		}

		//TODO: remove (dbg)

		let comp_statements_dbg = comp_statements
			.iter()
			.map(|x| x.to_string())
			.collect::<Vec<_>>()
			.join("\n");

		println!("{comp_statements_dbg}\n");

		Ok(CompFunc {
			comp_statements,
			signature: self.signature.clone(),
		})
	}
}
