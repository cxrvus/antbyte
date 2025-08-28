use anyhow::{Result, anyhow};

use super::CompFunc;

use crate::ant::{
	compiler::CompStatement,
	world::parser::{Func, Signature},
};

pub(super) fn compile_funcs(funcs: Vec<Func>) -> Result<Vec<CompFunc>> {
	let mut comp_funcs = vec![];

	for func in funcs.into_iter() {
		func.signature.validate()?;

		println!("{}:", func.signature.name);

		let comp_func = func.compile(&comp_funcs)?;

		comp_funcs.push(comp_func);
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
	fn compile(&self, comp_funcs: &Vec<CompFunc>) -> Result<CompFunc> {
		if comp_funcs.iter().any(|f| f.signature == self.signature) {
			return Err(anyhow!(
				"overload with signature {:?} already exists",
				self.signature
			));
		}

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
						if flat_statement.assignees.len() != 1 {
							return Err(anyhow!(
								"the result of an OR may only be assigned to a single assignee"
							));
						}

						comp_statements.push(flat_statement.into());
					}
					_ => {
						func_index += 1;
						let expanded = flat_statement.expand_call(comp_funcs, func_index)?;
						comp_statements.extend(expanded);
					}
				}
			}

			println!()
		}

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
