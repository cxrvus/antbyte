use anyhow::{Result, anyhow};

use crate::ant::{
	compiler::{Compiler, FlatStatement, ParamValue},
	world::parser::{Expression, Func, Signature, Statement},
};

impl Statement {
	pub(super) fn flatten(&self, start_index: &mut u32) -> Vec<FlatStatement> {
		let mut flat_statements = expand_expression(&self.expression, start_index);
		flat_statements.last_mut().unwrap().assignees = self.assignees.clone();
		flat_statements
	}
}

#[inline]
fn format_index(index: u32) -> String {
	format!("_exp{index:02}")
}

fn expand_expression(exp: &Expression, index: &mut u32) -> Vec<FlatStatement> {
	let mut flat_statements = vec![];

	let (func, params) = if let Some(parameters) = &exp.parameter_values {
		let mut params = vec![];

		for sub_exp in parameters {
			if sub_exp.parameter_values.is_some() {
				flat_statements.extend(expand_expression(sub_exp, index));

				params.push(ParamValue {
					sign: sub_exp.sign,
					target: format_index(*index - 1),
				});
			} else {
				params.push(ParamValue {
					sign: sub_exp.sign,
					target: sub_exp.ident.clone(),
				});
			}
		}

		(exp.ident.clone(), params)
	} else {
		let params = vec![ParamValue {
			sign: exp.sign,
			target: exp.ident.clone(),
		}];

		("or".to_string(), params)
	};

	flat_statements.push(FlatStatement {
		func,
		assignees: vec![format_index(*index)],
		sign: exp.sign,
		params,
	});

	*index += 1;
	flat_statements
}

impl Compiler {
	pub(super) fn validate_statements(
		&self,
		flat_statements: &Vec<FlatStatement>,
		func: &Func,
	) -> Result<()> {
		let Signature {
			in_params,
			out_params,
		} = &func.signature;

		for flat_statement in flat_statements {
			for params in flat_statement.params.iter() {
				let target = &params.target;

				let is_an_in_param = in_params.contains(target);
				let is_declared =
					is_an_in_param || flat_statements.iter().any(|x| x.assignees.contains(target));

				if !is_declared {
					let error = if self.norm_funcs.contains_key(target) {
						anyhow!("'{target}' is a func, not a value")
					} else if out_params.contains(target) {
						anyhow!("'{target}' is an out-param, not a value")
					} else {
						anyhow!("unknown identifier: '{target}'")
					};

					return Err(error);
				}
			}
		}

		Ok(())
	}
}
