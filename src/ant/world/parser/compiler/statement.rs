use anyhow::{Result, anyhow};

use crate::ant::{
	compiler::{FlatStatement, ParamValue},
	world::parser::{Expression, Statement},
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
}
