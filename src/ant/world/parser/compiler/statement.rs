use std::fmt::{self, Display};

use crate::ant::{
	compiler::{CompStatement, FuncCall, ParamValue},
	world::parser::{Expression, Statement},
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
		format!("_{index}")
	}

	fn expand(&self, index: &mut u32) -> Vec<FuncCall> {
		let mut func_calls = vec![];

		let (func, params) = if let Some(parameters) = &self.params {
			let mut params = vec![];

			for sub_exp in parameters {
				if sub_exp.params.is_some() {
					func_calls.extend(sub_exp.expand(index));
					params.push(ParamValue::target(Self::format_index(*index - 1)));
				} else {
					params.push(ParamValue {
						sign: sub_exp.sign,
						target: sub_exp.ident.clone(),
					});
				}
			}

			(self.ident.clone(), params)
		} else {
			let params = vec![ParamValue::target(self.ident.clone())];

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

impl Display for CompStatement {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self { assignee, params } = self;

		let params = params
			.iter()
			.map(|param| param.to_string())
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "{assignee} <- {params};")
	}
}
