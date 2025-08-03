use super::{
	CircuitType, ParsedWorld, Parser, Setting, Statement, circuit_parser::parse_circuit,
	token::Token,
};
use crate::ant::archetype::AntType;
use anyhow::{Result, anyhow};

pub fn parse_world(parser: &mut Parser) -> Result<ParsedWorld> {
	let mut statements: Vec<Statement> = vec![];

	loop {
		let statement = match parser.next_token() {
			Token::Ident(ident) => ident,
			Token::EndOfFile => break,
			// fixme: better error handling - parsing goes on even if statement is invalid
			other => return Err(Parser::unexpected(other, "statement")),
		};

		let ident = parser.next_ident()?;

		match parser.next_token() {
			Token::Assign => {}
			other => return Err(Parser::unexpected(other, "'='")),
		};

		if statement.as_str() == "set" {
			let setting = parse_setting(parser, ident)?;
			statements.push(Statement::Set(setting));
		} else if let Some(circuit_type) = match statement.as_str() {
			"queen" => Some(CircuitType::Ant(AntType::Queen)),
			"worker" => Some(CircuitType::Ant(AntType::Worker)),
			"fn" => Some(CircuitType::Sub),
			_ => None,
		} {
			let circuit = parse_circuit(parser, ident, circuit_type)?;
			statements.push(Statement::Declare(circuit));
		} else {
			return Err(anyhow!("invalid statement: {statement}"));
		}
	}

	let world = ParsedWorld { statements };

	Ok(dbg!(world))
}

fn parse_setting(parser: &mut Parser, key: String) -> Result<Setting> {
	parser.assume_next(Token::Assign);
	let value = parser.next_token();
	parser.expect_next(Token::Semicolon)?;

	match value {
		Token::Ident(_) | Token::Number(_) => Ok(Setting { key, value }),
		other => Err(Parser::unexpected(other, "value (identifier or number)")),
	}
}
