use std::fmt::{self, Display};

use anyhow::{Result, anyhow};

use crate::ant::{
	compiler::{CompFunc, CompStatement, FuncCall, ParamValue},
	world::parser::{Expression, Signature, Statement},
};

impl Statement {
	pub(super) fn expand_expression(&self, start_index: &mut u32) -> Vec<FuncCall> {
		let mut func_calls = self.expression.expand(start_index);
		let mut last_func_call = func_calls.pop().unwrap();
		let mut stm_assignees = self.assignees.clone();

		if last_func_call.assignees[0].sign {
			stm_assignees.iter_mut().for_each(|f| f.invert());
		}

		last_func_call.assignees = stm_assignees;
		func_calls.push(last_func_call);

		func_calls
	}
}

impl Expression {
	#[inline]
	fn format_index(index: u32) -> String {
		format!("_exp{index:02}")
	}

	fn expand(&self, index: &mut u32) -> Vec<FuncCall> {
		let mut func_calls = vec![];

		let (func, params) = if let Some(parameters) = &self.params {
			let mut params = vec![];

			for sub_exp in parameters {
				if sub_exp.params.is_some() {
					func_calls.extend(sub_exp.expand(index));

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

		let assignee = ParamValue {
			target: Self::format_index(*index),
			sign: self.sign,
		};

		func_calls.push(FuncCall {
			func,
			assignees: vec![assignee],
			params,
		});

		*index += 1;
		func_calls
	}
}

impl FuncCall {
	/// transform AND into OR ([DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws))
	pub(super) fn resolve_and_gate(&mut self) {
		if self.func == "and" {
			self.params.iter_mut().for_each(|param| param.invert());
			self.assignees.iter_mut().for_each(|asg| asg.invert());
			self.func = "or".into();
		}
	}

	pub(super) fn expand_call(
		&self,
		comp_funcs: &[CompFunc],
		func_index: u32,
	) -> Result<Vec<CompStatement>> {
		let called_func = self.get_overload(comp_funcs)?;

		let signature = &called_func.signature;
		let var_prefix = format!("_{}{func_index:02}", self.func);
		let mut expanded_statements = vec![];

		for mut func_stm in called_func.comp_statements.clone() {
			if let Some(assignee_index) = signature
				.assignees
				.iter()
				.position(|asg_target| *asg_target == func_stm.assignee.target)
			{
				// assignee represents a function assignee
				let call_assignee = &self.assignees[assignee_index];

				func_stm.assignee = ParamValue {
					sign: func_stm.assignee.sign ^ call_assignee.sign,
					target: call_assignee.target.clone(),
				}
			} else {
				// assignee represents a variable
				func_stm.assignee.target = var_prefix.clone() + &func_stm.assignee.target;
			}

			for func_param in func_stm.params.iter_mut() {
				if let Some(param_index) = signature
					.params
					.iter()
					.position(|param_target| *param_target == func_param.target)
				{
					// value targets a function parameter
					let call_param = &self.params[param_index];

					*func_param = ParamValue {
						sign: func_param.sign ^ call_param.sign,
						target: call_param.target.clone(),
					};
				} else {
					// value targets a variable
					func_param.target = var_prefix.clone() + &func_param.target;
				}
			}

			expanded_statements.push(func_stm);
		}

		Ok(expanded_statements)
	}

	fn get_overload<'a>(&self, comp_funcs: &'a [CompFunc]) -> Result<&'a CompFunc> {
		if !comp_funcs.iter().any(|f| f.signature.name == self.func) {
			return Err(anyhow!("unknown function: {}", self.func));
		}

		comp_funcs
			.iter()
			.find(|f| {
				let Signature {
					name,
					params,
					assignees,
				} = &f.signature;

				name == &self.func
					&& params.len() == self.params.len()
					&& assignees.len() == self.assignees.len()
			})
			.ok_or(anyhow!(
				"no overload found for function '{}'\nwith {} parameters and {} assignees",
				self.func,
				self.params.len(),
				self.assignees.len()
			))
	}
}

impl From<FuncCall> for CompStatement {
	fn from(func_call: FuncCall) -> Self {
		#[rustfmt::skip]
		let FuncCall { assignees, params, func } = func_call;

		assert_eq!(func, "or", "func must be 'or' \nfound '{func}'");

		assert_eq!(
			assignees.len(),
			1,
			"FuncCall must have exactly one left-hand-side value\nfound {assignees:?})",
		);

		Self {
			assignee: assignees[0].clone(),
			params: params.clone(),
		}
	}
}

impl Display for CompStatement {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let sign = sign_to_str(self.assignee.sign);

		let params = self
			.params
			.iter()
			.map(|param| sign_to_str(param.sign).to_string() + &param.target)
			.collect::<Vec<_>>()
			.join(", ");

		let assignee = &self.assignee.target;

		write!(f, "{sign}{assignee} <- {params};")
	}
}

fn sign_to_str(sign: bool) -> &'static str {
	match sign {
		false => "",
		true => "!",
	}
}
