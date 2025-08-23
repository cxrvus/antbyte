use super::{Parser, Token};
use crate::ant::{AntType, world::parser::ParsedWorld};
use anyhow::{Result, anyhow};

impl Parser {
	pub(super) fn parse_world(&mut self) -> Result<ParsedWorld> {
		let mut world = ParsedWorld::default();

		loop {
			let statement_type = match self.next_token() {
				Token::Ident(ident) => ident,
				Token::EndOfFile => break,
				// fixme: better error handling - parsing goes on even if statement is invalid
				other => return Err(Parser::unexpected(other, "global statement")),
			};

			let ident = self.next_ident()?;

			if statement_type == "set" {
				let (key, value) = parse_setting(self, ident)?;
				world.settings.push((key, value));
			} else if statement_type == "fn" {
				let func = self.parse_func()?;
				world.funcs.push((ident, func));
			} else if let Some(ant_type) = AntType::from_str(&statement_type) {
				let (func, ant) = self.parse_ant(ident.clone(), ant_type)?;
				world.funcs.push((ident, func));
				world.ants.push(ant);
			} else {
				return Err(anyhow!("invalid global statement: {statement_type}"));
			}
		}

		// dbg!(&statements);

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
