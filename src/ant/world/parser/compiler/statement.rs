use std::fmt::{self, Display};

use anyhow::{Result, anyhow};

use crate::ant::{
	compiler::{CompFuncs, CompStatement, FlatStatement, ParamValue},
	world::parser::{Expression, Signature, Statement},
};

impl Statement {
	pub(super) fn expand_expression(&self, start_index: &mut u32) -> Vec<FlatStatement> {
		let mut flat_statements = self.expression.expand(start_index);
		flat_statements.last_mut().unwrap().assignees = self.assignees.clone();
		flat_statements
	}
}

impl Expression {
	#[inline]
	fn format_index(index: u32) -> String {
		format!("_exp{index:02}")
	}

	fn expand(&self, index: &mut u32) -> Vec<FlatStatement> {
		let mut flat_statements = vec![];

		let (func, params) = if let Some(parameters) = &self.params {
			let mut params = vec![];

			for sub_exp in parameters {
				if sub_exp.params.is_some() {
					flat_statements.extend(sub_exp.expand(index));

					params.push(ParamValue {
						sign: sub_exp.sign,
						target: Self::format_index(*index - 1),
					});
				} else {
					params.push(ParamValue {
						sign: sub_exp.sign,
						target: sub_exp.ident.clone(),
					});
				}
			}

			(self.ident.clone(), params)
		} else {
			let params = vec![ParamValue {
				sign: self.sign,
				target: self.ident.clone(),
			}];

			("or".to_string(), params)
		};

		flat_statements.push(FlatStatement {
			func,
			assignees: vec![Self::format_index(*index)],
			sign: self.sign,
			params,
		});

		*index += 1;
		flat_statements
	}
}

impl FlatStatement {
	/// transform AND into OR ([DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws))
	pub(super) fn resolve_and_gate(&mut self) {
		if self.func == "and" {
			self.params
				.iter_mut()
				.for_each(|param| param.sign = !param.sign);

			self.sign = !self.sign;
			self.func = "or".into();
		}
	}

	pub(super) fn expand_call(
		&self,
		comp_funcs: &CompFuncs,
		func_name: &str,
		func_index: u32,
	) -> Result<Vec<CompStatement>> {
		let called_func = comp_funcs
			.get(func_name)
			.ok_or(anyhow!("unknown function: '{func_name}'"))?;

		let var_prefix = format!("_{func_name}{func_index:02}");
		let mut expanded_statements = vec![];
		let signature = &called_func.signature;

		validate_call(signature, self, func_name)?;

		for mut comp_statement in called_func.comp_statements.clone() {
			if let Some(out_param_index) = signature
				.out_params
				.iter()
				.position(|out_param| *out_param == comp_statement.assignee)
			{
				// assignee represents a function out-param
				comp_statement.assignee = self.assignees[out_param_index].clone();
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
					let in_param_value = &self.params[in_param_index];
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

fn validate_call(signature: &Signature, statement: &FlatStatement, func_name: &str) -> Result<()> {
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
	} else {
		Ok(())
	}
}

impl From<FlatStatement> for CompStatement {
	fn from(flat_statement: FlatStatement) -> Self {
		#[rustfmt::skip]
		let FlatStatement { assignees, sign, params, func } = flat_statement;

		assert_eq!(
			func, "or",
			"FlatStatement func must be 'or' \nfound '{func}'"
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

impl Display for CompStatement {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self {
			assignee,
			sign,
			params,
		} = self;

		let sign = sign_to_str(*sign);

		let params = params
			.iter()
			.map(|param| sign_to_str(param.sign).to_string() + &param.target)
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "{sign}{assignee} <- {params};")
	}
}

fn sign_to_str(sign: bool) -> &'static str {
	match sign {
		false => "",
		true => "!",
	}
}
