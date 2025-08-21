use crate::ant::{AntType, world::parser::Signature};

use super::{CircuitType, ParsedCircuit, Parser, Statement, Token};

use anyhow::{Ok, Result};

impl Parser {
	pub(super) fn parse_ant(&mut self, name: String, ant_type: AntType) -> Result<ParsedCircuit> {
		let statements = self.parse_statements()?;

		Ok(ParsedCircuit {
			name,
			statements,
			circuit_type: CircuitType::Ant(ant_type),
		})
	}

	pub(super) fn parse_func(&mut self, name: String) -> Result<ParsedCircuit> {
		self.expect_next(Token::Assign)?;

		let signature = self.parse_signature()?;
		let statements = self.parse_statements()?;

		Ok(ParsedCircuit {
			name,
			statements,
			circuit_type: CircuitType::Sub(signature),
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
