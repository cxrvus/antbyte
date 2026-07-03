use crate::parser::compiler::linker::{WorldImport, WorldImportMode};

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
					let path = self.next_str()?;
					let import = WorldImport {
						path,
						mode: WorldImportMode::Functions,
					};
					world.imports.push(import);
					self.expect_next(Token::Semicolon)?;
				}
				UseCfg => {
					let path = self.next_str()?;
					let import = WorldImport {
						path,
						mode: WorldImportMode::Config,
					};
					world.imports.push(import);
					self.expect_next(Token::Semicolon)?;
				}
				Set => {
					if self.assume_next(Token::BraceLeft).is_some() {
						while self.assume_next(Token::BraceRight).is_none() {
							let (key, value) = self.parse_setting()?;
							world.settings.push((key, value));
						}
					} else {
						let (key, value) = self.parse_setting()?;
						world.settings.push((key, value));
					}
				}
				Fn => {
					let name = self.next_ident()?;
					let func = self
						.parse_func(name.clone())
						.with_context(|| format!("in function '{name}'!"))?;
					world.funcs.push(func);
				}
				Ant => {
					let (id, name) =
						if let Some(Token::Number(id)) = self.assume_next(Token::Number(0)) {
							let name = format!("_ant_0x{id:02x}");
							(id, name)
						} else if let Some(Token::Bit(id)) = self.assume_next(Token::Bit(false)) {
							let id = id.into();
							let name = format!("_ant_0x{id:02x}");
							(id, name)
						} else {
							let name = self.next_ident()?;
							self.expect_next(Token::Assign)?;
							let id = self.next_number()?.unwrap_or_default();
							(id, name)
						};

					let (func, ant) = self
						.parse_ant(name.clone(), id)
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

	pub fn parse_setting(&mut self) -> Result<(String, Token)> {
		let key = self.next_ident()?;
		self.expect_next(Token::Assign)?;
		let value = self.next_token();
		self.expect_next(Token::Semicolon)?;

		Ok((key, value))
	}
}
