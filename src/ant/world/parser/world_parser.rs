use super::{GlobalStatement, ParsedWorld, Parser, Token};
use crate::ant::AntType;
use anyhow::{Result, anyhow};

impl Parser {
	pub(super) fn parse_world(&mut self) -> Result<ParsedWorld> {
		let mut global_statements: Vec<GlobalStatement> = vec![];

		loop {
			let statement_type = match self.next_token() {
				Token::Ident(ident) => ident,
				Token::EndOfFile => break,
				// fixme: better error handling - parsing goes on even if statement is invalid
				other => return Err(Parser::unexpected(other, "global statement")),
			};

			let ident = self.next_ident()?;

			if statement_type.as_str() == "set" {
				let (key, value) = parse_setting(self, ident)?;
				global_statements.push(GlobalStatement::Set(key, value));
			} else if let Some(circuit) = match statement_type.as_str() {
				"queen" => Some(self.parse_ant(ident, AntType::Queen)),
				"worker" => Some(self.parse_ant(ident, AntType::Worker)),
				"fn" => Some(self.parse_func(ident)),
				_ => None,
			} {
				global_statements.push(GlobalStatement::Declare(circuit?));
			} else {
				return Err(anyhow!("invalid global statement: {statement_type}"));
			}
		}

		let world = ParsedWorld {
			statements: global_statements,
		};

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
