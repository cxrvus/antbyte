use crate::ant::world::parser::{AntFunc, Func, Signature};

use super::{Parser, Statement, Token};

use anyhow::{Result, anyhow};

impl Parser {
	pub(super) fn parse_ant(&mut self, name: &str) -> Result<(Func, AntFunc)> {
		let target_id = if self.assume_next(Token::At) {
			let target_id = self.next_token();
			if let Token::Number(target_id) = target_id {
				Some(target_id as u8)
			} else {
				return Err(anyhow!("expected Ant target ID after '@'"));
			}
		} else {
			None
		};

		let ant = AntFunc {
			target_func: name.into(),
			target_id,
		};

		let statements = self.parse_statements()?;

		let func = Func {
			statements,
			signature: Default::default(),
		};

		Ok((func, ant))
	}

	pub(super) fn parse_func(&mut self) -> Result<Func> {
		self.expect_next(Token::Assign)?;

		let signature = self.parse_signature()?;
		let statements = self.parse_statements()?;

		Ok(Func {
			statements,
			signature,
		})
	}

	fn parse_signature(&mut self) -> Result<Signature> {
		// idea: require parentheses like in JS
		let in_params = self.next_ident_list()?;

		self.expect_next(Token::Arrow)?;

		let out_params: Vec<String> = self.next_ident_list()?;

		Ok(Signature {
			in_params,
			out_params,
		})
	}

	fn parse_statements(&mut self) -> Result<Vec<Statement>> {
		self.expect_next(Token::BraceLeft)?;

		let mut statements: Vec<Statement> = vec![];

		loop {
			let assignees = self.next_ident_list()?;

			self.expect_next(Token::Assign)?;

			let expression = self.parse_next_exp()?;

			statements.push(Statement {
				assignees,
				expression,
			});

			self.expect_next(Token::Semicolon)?;

			if self.assume_next(Token::BraceRight) {
				break;
			}
		}

		Ok(statements)
	}
}
