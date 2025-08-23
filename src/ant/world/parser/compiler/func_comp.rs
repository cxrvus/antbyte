use std::collections::HashMap;

use anyhow::{Result, anyhow};

use super::{FlatStatement, NormFunc};

use crate::{
	ant::{
		compiler::{Compiler, NormStatement},
		world::parser::{Func, FuncType, Signature},
	},
	util::find_dupe,
};

impl Compiler {
	pub(super) fn normalize_funcs(funcs: Vec<Func>) -> Result<HashMap<String, NormFunc>> {
		let mut compiler = Compiler::default();

		for func in funcs.into_iter() {
			if let FuncType::Sub(signature) = &func.func_type {
				signature.validate()?;
			}

			let func_name = func.name.clone();
			let norm_func = compiler.normalize_func(func)?;

			if compiler.0.insert(func_name.clone(), norm_func).is_some() {
				return Err(anyhow!("func name '{func_name}' used more than once"));
			}
		}

		Ok(compiler.0)
	}

	fn normalize_func(&self, func: Func) -> Result<NormFunc> {
		let mut exp_index = 0;
		let mut func_index = 0;
		let mut norm_statements: Vec<NormStatement> = vec![];

		for statement in func.statements.iter() {
			exp_index += 1;

			let mut flat_statements = statement.flatten(&mut exp_index);

			resolve_and_gates(&mut flat_statements);

			self.validate_statements(&flat_statements, &func)?;

			for flat_statement in flat_statements {
				match flat_statement.call.as_str() {
					"or" => {
						if flat_statement.assignees.len() != 1 {
							return Err(anyhow!(
								"the result of an OR may only be assigned to a single assignee"
							));
						}

						norm_statements.push(flat_statement.into());
					}
					call => {
						func_index += 1;
						let expanded = self.expand_func_call(call, &flat_statement, func_index)?;
						norm_statements.extend(expanded);
					}
				}
			}

			println!("\n\n\n") //TODO: remove (dbg)
		}

		let all_assignees: Vec<_> = norm_statements.iter().map(|stm| &stm.assignee).collect();

		if let Some(dupe_assignee) = find_dupe(&all_assignees) {
			return Err(anyhow!(
				"identifier '{dupe_assignee}' can not be assigned to more than once"
			));
		}

		dbg!(&norm_statements);

		Ok(NormFunc {
			norm_statements,
			original: func,
		})
	}

	fn expand_func_call(
		&self,
		call: &str,
		flat_statement: &FlatStatement,
		func_index: u32,
	) -> Result<Vec<NormStatement>> {
		let called_func = self
			.0
			.get(call)
			.ok_or(anyhow!("unknown function: '{call}'"))?;

		match &called_func.original.func_type {
			FuncType::Ant(ant_type) => {
				Err(anyhow!("func '{call}' is a {ant_type:?}, not a function"))
			}
			FuncType::Sub(signature) => {
				let var_prefix = format!("_{call}{func_index:02}");

				let mut expanded_statements = vec![];

				validate_call(signature, &called_func.original.name, flat_statement)?;

				for mut norm_statement in called_func.norm_statements.clone() {
					if let Some(out_param_index) = signature
						.out_params
						.iter()
						.position(|out_param| *out_param == norm_statement.assignee)
					{
						// assignee represents a function out-param
						norm_statement.assignee = flat_statement.assignees[out_param_index].clone();
					} else {
						// assignee represents a variable
						norm_statement.assignee = var_prefix.clone() + &norm_statement.assignee;
					}

					for param in norm_statement.params.iter_mut() {
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

					expanded_statements.push(norm_statement);
				}

				Ok(expanded_statements)
			}
		}
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
		.filter(|stm| stm.call == "and")
		.for_each(|stm| {
			stm.params
				.iter_mut()
				.for_each(|param| param.sign = !param.sign);

			stm.sign = !stm.sign;
			stm.call = "or".into();
		});
}
