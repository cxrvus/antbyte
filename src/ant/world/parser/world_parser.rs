use super::{CircuitType, ParsedWorld, Parser, Statement, Token};
use crate::ant::AntType;
use anyhow::{Result, anyhow};

impl Parser {
	pub(super) fn parse_world(&mut self) -> Result<ParsedWorld> {
		let mut statements: Vec<Statement> = vec![];

		loop {
			let statement = match self.next_token() {
				Token::Ident(ident) => ident,
				Token::EndOfFile => break,
				// fixme: better error handling - parsing goes on even if statement is invalid
				other => return Err(Parser::unexpected(other, "statement")),
			};

			let ident = self.next_ident()?;

			match self.next_token() {
				Token::Assign => {}
				other => return Err(Parser::unexpected(other, "'='")),
			};

			if statement.as_str() == "set" {
				let (key, value) = parse_setting(self, ident)?;
				statements.push(Statement::Set(key, value));
			} else if let Some(circuit_type) = match statement.as_str() {
				"queen" => Some(CircuitType::Ant(AntType::Queen)),
				"worker" => Some(CircuitType::Ant(AntType::Worker)),
				"fn" => Some(CircuitType::Sub),
				_ => None,
			} {
				let circuit = self.parse_circuit(ident, circuit_type)?;
				statements.push(Statement::Declare(circuit));
			} else {
				return Err(anyhow!("invalid statement: {statement}"));
			}
		}

		let world = ParsedWorld { statements };

		// dbg!(&world);

		Ok(world)
	}
}

fn parse_setting(parser: &mut Parser, key: String) -> Result<(String, Token)> {
	parser.assume_next(Token::Assign);
	let value = parser.next_token();
	parser.expect_next(Token::Semicolon)?;

	match value {
		Token::Ident(_) | Token::Number(_) => Ok((key, value)),
		other => Err(Parser::unexpected(other, "value (identifier or number)")),
	}
}
