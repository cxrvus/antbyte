use super::{Assignment, CircuitType, ParsedCircuit, Parser, Token};

use anyhow::Result;

impl Parser {
	pub(super) fn parse_circuit(
		&mut self,
		name: String,
		circuit_type: CircuitType,
	) -> Result<ParsedCircuit> {
		let inputs = self.next_ident_list()?;

		self.expect_next(Token::Arrow)?;

		let outputs: Vec<String> = self.next_ident_list()?;

		self.expect_next(Token::BraceLeft)?;

		let mut assignments: Vec<Assignment> = vec![];

		loop {
			let lhs = self.next_ident_list()?;

			self.expect_next(Token::Assign)?;

			let rhs = self.parse_next_exp()?;
			assignments.push(Assignment { lhs, rhs });

			self.expect_next(Token::Semicolon)?;

			if self.assume_next(Token::BraceRight) {
				break;
			}
		}

		let circuit = ParsedCircuit {
			name,
			circuit_type,
			inputs,
			outputs,
			assignments,
		};

		Ok(circuit)
	}
}
