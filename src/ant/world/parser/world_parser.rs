use super::{Parser, Token};
use crate::ant::world::parser::{ParsedWorld, token::Keyword};
use anyhow::Result;

impl Parser {
	pub(super) fn parse_world(&mut self) -> Result<ParsedWorld> {
		let mut world = ParsedWorld::default();

		loop {
			let keyword = match self.next_token() {
				Token::Keyword(keyword) => keyword,
				Token::EndOfFile => break,
				other => return Err(Parser::unexpected(other, "global statement")),
			};

			let ident = self.next_ident()?;

			use Keyword::*;

			match keyword {
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

		Ok(world)
	}

	pub(super) fn parse_setting(&mut self, key: String) -> Result<(String, Token)> {
		self.assume_next(Token::Assign);
		let value = self.next_token();
		self.expect_next(Token::Semicolon)?;

		Ok((key, value))
	}
}
