pub mod compiler;

mod expression_parser;
mod func_parser;
mod token;
mod world_parser;

use self::token::Token;
use crate::ant::AntType;
use anyhow::{Error, Ok, Result, anyhow};

#[derive(Debug)]
enum GlobalStatement {
	Set(String, Token),
	Declare(Func),
}

#[derive(Debug)]
struct Func {
	name: String,
	func_type: FuncType,
	statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
struct Signature {
	in_params: Vec<String>,
	out_params: Vec<String>,
}

#[rustfmt::skip]
#[derive(Debug, Clone)]
enum FuncType { Ant(AntType), Sub(Signature) }

#[derive(Debug)]
struct Statement {
	assignees: Vec<String>,
	expression: Expression,
}

#[derive(Debug)]
struct Expression {
	ident: String,
	sign: bool,
	/// is a function if Some, else variable
	parameter_values: Option<Vec<Self>>,
}

#[derive(Default)]
struct Parser {
	tokens: Vec<Token>,
}

impl Parser {
	fn new(code: String) -> Self {
		let mut tokens = Token::tokenize(code);
		tokens.reverse();
		Self { tokens }
	}

	#[inline]
	fn next_token(&mut self) -> Token {
		self.tokens.pop().unwrap_or_default()
	}

	#[inline]
	fn unexpected(unexpected: Token, expected: &str) -> Error {
		anyhow!("unexpected token: {unexpected:?}, expected {expected}")
	}

	fn expect(actual: Token, expected: Token) -> Result<()> {
		if actual != expected {
			Err(Self::unexpected(actual, format!("{expected:?}").as_str()))
		} else {
			Ok(())
		}
	}

	#[inline]
	fn expect_next(&mut self, expected: Token) -> Result<()> {
		Self::expect(self.next_token(), expected)
	}

	fn assume_next(&mut self, expected: Token) -> bool {
		let actual = self.next_token();
		if actual == expected {
			true
		} else {
			self.tokens.push(actual.clone());
			false
		}
	}

	fn next_ident(&mut self) -> Result<String> {
		let token = self.next_token();

		if let Token::Ident(ident) = token {
			Ok(ident)
		} else {
			Err(Self::unexpected(token, "identifier"))
		}
	}

	fn next_ident_list(&mut self) -> Result<Vec<String>> {
		let identifiers = if self.assume_next(Token::ParenthesisLeft) {
			if self.assume_next(Token::ParenthesisRight) {
				vec![]
			} else {
				let mut identifiers: Vec<String> = vec![];
				let mut expect_ident = true;

				loop {
					if expect_ident {
						let ident = self.next_ident()?;
						identifiers.push(ident);
					} else if !self.assume_next(Token::Comma) {
						break;
					}

					expect_ident = !expect_ident;
				}

				self.expect_next(Token::ParenthesisRight)?;
				identifiers
			}
		} else {
			vec![self.next_ident()?]
		};

		Ok(identifiers)
	}
}
