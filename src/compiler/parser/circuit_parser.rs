use super::{
	Assignment, CircuitType, ParsedCircuit, Parser, expression_parser::parse_next_exp, token::Token,
};

use anyhow::Result;

pub fn parse_circuit(parser: &mut Parser, circuit_type: CircuitType) -> Result<ParsedCircuit> {
	let inputs = parser.next_ident_list()?;

	parser.expect_next(Token::Arrow)?;

	let outputs: Vec<String> = parser.next_ident_list()?;

	parser.expect_next(Token::BraceLeft)?;

	let mut assignments: Vec<Assignment> = vec![];

	loop {
		let lhs = parser.next_ident_list()?;

		parser.expect_next(Token::Assign)?;

		let rhs = parse_next_exp(parser)?;
		assignments.push(Assignment { lhs, rhs });

		parser.expect_next(Token::Semicolon)?;

		if parser.assume_next(Token::BraceRight) {
			break;
		}
	}

	let circuit = ParsedCircuit {
		circuit_type,
		used_inputs: inputs,
		used_outputs: outputs,
		assignments,
	};

	Ok(circuit)
}
