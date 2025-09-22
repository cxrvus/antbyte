use crate::ant::{
	compiler::{CompFunc, CompStatement, FuncCall},
	world::parser::{ParamValue, SignatureSpec, token::Token},
};

use anyhow::{Result, anyhow, bail};

impl FuncCall {
	pub(super) fn expand_call(
		&self,
		comp_funcs: &[CompFunc],
		func_index: u32,
	) -> Result<Vec<CompStatement>> {
		let called_func = SignatureSpec::from(self).get_overload(comp_funcs)?;
		let called_assigns = &called_func.signature.assignees;
		let called_params = &called_func.signature.params;

		let var_prefix = format!("_{}_{func_index}_", self.func);
		let mut comp_statements = vec![];

		comp_statements.extend(self.params.clone().into_iter().zip(called_params).map(
			|(param_value, param)| CompStatement {
				assignee: ParamValue::target(format!("{var_prefix}{param}")),
				params: vec![param_value],
			},
		));

		for mut called_func_stm in called_func.comp_statements.clone() {
			prefix_var(&mut called_func_stm.assignee, &var_prefix);

			for stm_param in called_func_stm.params.iter_mut() {
				prefix_var(stm_param, &var_prefix);
			}

			comp_statements.push(called_func_stm);
		}

		comp_statements.extend(self.assignees.clone().into_iter().zip(called_assigns).map(
			|(assignee_value, assignee)| CompStatement {
				assignee: assignee_value,
				params: vec![ParamValue::target(format!("{var_prefix}{assignee}"))],
			},
		));

		Ok(comp_statements)
	}
}

#[inline]
fn prefix_var(value: &mut ParamValue, var_prefix: &str) {
	if !Token::is_uppercase_ident(&value.target) {
		value.target = format!("{var_prefix}{}", &value.target);
	}
}

impl<'a> SignatureSpec<'a> {
	pub(super) fn get_overload<'b>(&self, comp_funcs: &'b [CompFunc]) -> Result<&'b CompFunc> {
		if !comp_funcs.iter().any(|f| f.signature.name == self.name) {
			bail!("unknown function: '{}'", self.name);
		}

		comp_funcs
			.iter()
			.find(|f| SignatureSpec::from(&f.signature) == *self)
			.ok_or(anyhow!("no overload found for [{self}]"))
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

impl<'a> From<&'a FuncCall> for SignatureSpec<'a> {
	fn from(func_call: &'a FuncCall) -> Self {
		Self {
			name: &func_call.func,
			param_count: func_call.params.len(),
			assignee_count: func_call.assignees.len(),
		}
	}
}
