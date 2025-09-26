use super::{Keyword, ParsedWorld, Parser, Token};
use anyhow::{Context, Result};

impl Parser {
	pub(super) fn parse_world(&mut self) -> Result<ParsedWorld> {
		let mut world = ParsedWorld::default();

		loop {
			let keyword = match self.next_token() {
				Token::Keyword(keyword) => keyword,
				Token::EndOfFile => break,
				other => return Err(Parser::unexpected(other, "instruction")),
			};

			use Keyword::*;

			match keyword {
				Use => {
					let token = self.next_token();

					if let Token::String(import) = token {
						world.imports.push(import);
					} else {
						return Err(Self::unexpected(token, "path to import (string)"));
					}

					self.expect_next(Token::Semicolon)?;
				}
				Set => {
					let (key, value) = self.parse_setting()?;
					world.settings.push((key, value));
				}
				Fn => {
					let name = self.next_ident()?;
					let func = self
						.parse_func(name.clone())
						.with_context(|| format!("in function '{name}'!"))?;
					world.funcs.push(func);
				}
				Ant => {
					let name = self.next_ident()?;
					let (func, ant) = self
						.parse_ant(name.clone())
						.with_context(|| format!("in ant '{name}'!"))?;
					world.funcs.push(func);
					world.ants.push(ant);
				}
				NoStd => {
					world.no_std = true;
					self.expect_next(Token::Semicolon)?;
				}
			};
		}

		Ok(world)
	}

	pub(super) fn parse_setting(&mut self) -> Result<(String, Token)> {
		let key = self.next_ident()?;
		self.assume_next(Token::Assign);
		let value = self.next_token();
		self.expect_next(Token::Semicolon)?;

		Ok((key, value))
	}
}
