use crate::ant::{
	compiler::{CompFunc, CompStatement, FuncCall},
	world::parser::{ParamValue, Signature, token::Token},
};

use anyhow::{Result, anyhow};

impl FuncCall {
	/// transforms AND into OR ([DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws))
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

		let var_prefix = format!("_{}_{func_index}", self.func);
		let mut expanded_statements = vec![];

		for mut called_func_stm in called_func.comp_statements.clone() {
			// resolve assignee
			Self::resolve_param(
				&mut called_func_stm.assignee,
				&self.assignees,
				&called_func.signature.assignees,
				&var_prefix,
			);

			// resolve parameters
			for func_param in called_func_stm.params.iter_mut() {
				Self::resolve_param(
					func_param,
					&self.params,
					&called_func.signature.params,
					&var_prefix,
				);
			}

			expanded_statements.push(called_func_stm);
		}

		Ok(expanded_statements)
	}

	/// resolves a statement parameter / assignee in by either mapping it to a func parameter / assignee
	/// or prefixing it with the variable prefix if it's a variable
	fn resolve_param(
		func_param: &mut ParamValue,
		call_params: &[ParamValue],
		signature_targets: &[String],
		var_prefix: &str,
	) {
		if Token::is_uppercase_ident(&func_param.target) {
			// value targets a peripheral => do nothing
		} else if let Some(call_value) = signature_targets
			.iter()
			.position(|target| *target == func_param.target)
			.map(|i| &call_params[i])
		{
			// value targets a function parameter / assignee
			*func_param = ParamValue {
				sign: func_param.sign ^ call_value.sign,
				target: call_value.target.clone(),
			};
		} else {
			// value targets a variable
			func_param.target = var_prefix.to_string() + &func_param.target;
		}
	}

	pub(super) fn get_overload<'a>(&self, comp_funcs: &'a [CompFunc]) -> Result<&'a CompFunc> {
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

		debug_assert_eq!(func, "or", "func must be 'or' \nfound '{func}'");

		debug_assert_eq!(
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
