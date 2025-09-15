use anyhow::{Result, bail};

use super::CompFunc;

use crate::ant::{compiler::CompStatement, world::parser::Func};

pub(super) fn compile_funcs(funcs: Vec<Func>) -> Result<Vec<CompFunc>> {
	let mut comp_funcs = vec![];

	for func in funcs.into_iter() {
		let comp_func = func.compile(&comp_funcs)?;
		comp_funcs.push(comp_func);
	}

	Ok(comp_funcs)
}

impl Func {
	fn compile(&self, comp_funcs: &[CompFunc]) -> Result<CompFunc> {
		if comp_funcs.iter().any(|f| f.signature == self.signature) {
			bail!(
				"overload with signature {:?} already exists",
				self.signature
			);
		}

		let mut exp_index = 0;
		let mut func_index = 0;
		let mut comp_statements: Vec<CompStatement> = vec![];

		for statement in self.statements.iter() {
			let mut func_calls = statement.expand_expression(&mut exp_index);

			func_calls.iter_mut().for_each(|stm| stm.resolve_and_gate());

			for func_call in func_calls {
				match func_call.func.as_str() {
					"or" => {
						if func_call.assignees.len() != 1 {
							bail!("the result of an OR may only be assigned to a single assignee");
						}

						comp_statements.push(func_call.into());
					}
					_ => {
						func_index += 1;
						let expanded = func_call.expand_call(comp_funcs, func_index)?;
						comp_statements.extend(expanded);
					}
				}
			}

			exp_index += 1;
		}

		comp_statements.retain(|stm| stm.assignee.target != "_");

		Ok(CompFunc {
			comp_statements,
			signature: self.signature.clone(),
		})
	}
}
