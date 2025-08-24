use anyhow::{Result, anyhow};

use super::{CompFunc, FlatStatement};

use crate::{
	ant::{
		compiler::{CompStatement, Compiler},
		world::parser::{Func, Signature},
	},
	util::find_dupe,
};

impl Compiler {
	pub(super) fn compile_funcs(&mut self, funcs: Vec<(String, Func)>) -> Result<()> {
		for (name, func) in funcs.into_iter() {
			func.signature.validate()?;

			let comp_func = self.compile_func(&name, func)?;

			if self.comp_funcs.insert(name.clone(), comp_func).is_some() {
				return Err(anyhow!("func name '{name}' used more than once"));
			}
		}

		Ok(())
	}

	fn compile_func(&self, name: &str, func: Func) -> Result<CompFunc> {
		let mut exp_index = 0;
		let mut func_index = 0;
		let mut comp_statements: Vec<CompStatement> = vec![];

		for statement in func.statements.iter() {
			exp_index += 1;

			let mut flat_statements = statement.flatten(&mut exp_index);

			resolve_and_gates(&mut flat_statements);

			self.validate_statements(&flat_statements, &func)?;

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
					func => {
						func_index += 1;
						let expanded = self.expand_func_call(func, &flat_statement, func_index)?;
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

		println!("{name}:\n{comp_statements_dbg}\n");

		Ok(CompFunc {
			comp_statements,
			signature: func.signature,
		})
	}

	fn expand_func_call(
		&self,
		func_name: &str,
		flat_statement: &FlatStatement,
		func_index: u32,
	) -> Result<Vec<CompStatement>> {
		let called_func = self
			.comp_funcs
			.get(func_name)
			.ok_or(anyhow!("unknown function: '{func_name}'"))?;

		let var_prefix = format!("_{func_name}{func_index:02}");
		let mut expanded_statements = vec![];
		let signature = &called_func.signature;

		validate_call(signature, func_name, flat_statement)?;

		for mut comp_statement in called_func.comp_statements.clone() {
			if let Some(out_param_index) = signature
				.out_params
				.iter()
				.position(|out_param| *out_param == comp_statement.assignee)
			{
				// assignee represents a function out-param
				comp_statement.assignee = flat_statement.assignees[out_param_index].clone();
			} else {
				// assignee represents a variable
				comp_statement.assignee = var_prefix.clone() + &comp_statement.assignee;
			}

			for param in comp_statement.params.iter_mut() {
				if let Some(in_param_index) = signature
					.in_params
					.iter()
					.position(|in_param| *in_param == param.target)
				{
					// value targets a function in-parameter
					let in_param_value = &flat_statement.params[in_param_index];
					param.target = in_param_value.target.clone();
					param.sign ^= in_param_value.sign;
				} else {
					// value targets a variable
					param.target = var_prefix.clone() + &param.target;
				}
			}

			expanded_statements.push(comp_statement);
		}

		Ok(expanded_statements)
	}
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

fn validate_call(signature: &Signature, func_name: &str, statement: &FlatStatement) -> Result<()> {
	let (in_count, out_count, param_val_count, assignee_count) = (
		signature.in_params.len(),
		signature.out_params.len(),
		statement.params.len(),
		statement.assignees.len(),
	);

	if param_val_count != in_count {
		Err(anyhow!(
			"function '{func_name}' has been given an invalid number of parameter values\nexpected {in_count}, got {param_val_count}"
		))
	} else if assignee_count != out_count {
		Err(anyhow!(
			"function '{func_name}' has been given an invalid number of assignees\nexpected {out_count}, got {assignee_count}"
		))
	} else if let Some(dupe_ident) = signature.in_params.iter().find(|in_param| {
		signature
			.out_params
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

/// transform AND into OR ([DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws))
fn resolve_and_gates(statements: &mut [FlatStatement]) {
	statements
		.iter_mut()
		.filter(|stm| stm.func == "and")
		.for_each(|stm| {
			stm.params
				.iter_mut()
				.for_each(|param| param.sign = !param.sign);

			stm.sign = !stm.sign;
			stm.func = "or".into();
		});
}
