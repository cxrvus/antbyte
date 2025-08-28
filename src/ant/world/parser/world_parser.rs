use super::{Parser, Token};
use crate::ant::world::parser::ParsedWorld;
use anyhow::{Error, Result, anyhow};

enum Declaration {
	Set,
	Fn,
	Ant,
}

impl TryFrom<String> for Declaration {
	type Error = Error;

	fn try_from(keyword: String) -> Result<Self> {
		match keyword.as_str() {
			"set" => Ok(Self::Set),
			"fn" => Ok(Self::Fn),
			"ant" => Ok(Self::Ant),
			_ => Err(anyhow!("invalid declaration keyword: {keyword}")),
		}
	}
}

impl Parser {
	pub(super) fn parse_world(&mut self) -> Result<ParsedWorld> {
		let mut world = ParsedWorld::default();

		loop {
			let declaration_keyword = match self.next_token() {
				Token::Ident(ident) => ident,
				Token::EndOfFile => break,
				// fixme: better error handling - parsing goes on even if statement is invalid
				other => return Err(Parser::unexpected(other, "global statement")),
			};

			let declaration = Declaration::try_from(declaration_keyword)?;
			let ident = self.next_ident()?;

			use Declaration::*;

			match declaration {
				Set => {
					let (key, value) = self.parse_setting(ident)?;
					world.settings.push((key, value));
				}
				Fn => {
					let func = self.parse_func(ident)?;
					world.funcs.push(func);
				}
				Ant => {
					let (func, ant) = self.parse_ant(ident)?;
					world.funcs.push(func);
					world.ants.push(ant);
				}
			};
		}

		// dbg!(&statements);

		Ok(world)
	}

	pub(super) fn parse_setting(&mut self, key: String) -> Result<(String, Token)> {
		self.assume_next(Token::Assign);
		let value = self.next_token();
		self.expect_next(Token::Semicolon)?;

		match value {
			Token::Ident(_) | Token::Number(_) => Ok((key, value)),
			other => Err(Parser::unexpected(other, "value (identifier or number)")),
		}
	}

	pub(super) fn is_declaration_keyword(ident: &String) -> bool {
		Declaration::try_from(ident.clone()).is_ok()
	}
}
