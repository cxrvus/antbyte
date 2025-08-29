use anyhow::{Result, anyhow};

use super::CompFunc;

use crate::{
	ant::{
		compiler::CompStatement,
		world::parser::{Func, Signature, token::Token},
	},
	util::find_dupe,
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
		self.validate_keywords()?;

		if let Some(collision) = self
			.params
			.iter()
			.find(|param| self.assignees.iter().any(|assignee| assignee == *param))
		{
			Err(anyhow!(
				"identifier '{collision}' used both as a parameter and an assignee"
			))
		} else if self.params.contains(&self.name) || self.assignees.contains(&self.name) {
			Err(anyhow!(
				"cannot use func name {} as parameter or assignee",
				self.name
			))
		} else if let Some(dupe) = find_dupe(&self.params) {
			Err(anyhow!("identifier {dupe} used for multiple parameters"))
		} else if let Some(dupe) = find_dupe(&self.assignees) {
			Err(anyhow!("identifier {dupe} used for multiple assignees"))
		} else {
			Ok(())
		}
	}

	fn validate_keywords(&self) -> Result<()> {
		let Signature {
			name,
			assignees,
			params,
		} = self;

		let mut idents = vec![name];
		idents.extend(params);
		idents.extend(assignees);

		for ident in idents {
			if Token::is_uppercase_ident(ident) {
				return Err(anyhow!(
					"may only use lower-case identifiers in function signatures\nfound '{ident}' in function '{name}'"
				));
			}
		}

		Ok(())
	}
}

impl Func {
	fn compile(&self, comp_funcs: &[CompFunc]) -> Result<CompFunc> {
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

			let mut func_calls = statement.expand_expression(&mut exp_index);

			func_calls.iter_mut().for_each(|stm| stm.resolve_and_gate());

			for func_call in func_calls {
				match func_call.func.as_str() {
					"or" => {
						if func_call.assignees.len() != 1 {
							return Err(anyhow!(
								"the result of an OR may only be assigned to a single assignee"
							));
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
