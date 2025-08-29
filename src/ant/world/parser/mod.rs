pub mod compiler;

mod expression_parser;
mod func_parser;
mod token;
mod world_parser;

use self::token::Token;
use anyhow::{Error, Ok, Result, anyhow};

#[derive(Debug, Default)]
struct ParsedWorld {
	settings: Vec<(String, Token)>,
	funcs: Vec<Func>,
	ants: Vec<AntFunc>,
}

#[derive(Debug)]
struct Func {
	signature: Signature,
	statements: Vec<Statement>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Signature {
	name: String,
	assignees: Vec<String>,
	params: Vec<String>,
}

#[derive(Debug)]
struct Statement {
	assignees: Vec<ParamValue>,
	expression: Expression,
}

#[derive(Debug, Clone)]
struct ParamValue {
	sign: bool,
	target: String,
}

impl ParamValue {
	#[inline]
	fn invert(&mut self) {
		self.sign = !self.sign;
	}
}

#[derive(Debug)]
struct Expression {
	ident: String,
	sign: bool,
	/// is a function if Some, else variable
	params: Option<Vec<Self>>,
}

#[derive(Debug)]
struct AntFunc {
	target_name: String,
	target_id: Option<u8>,
}

#[derive(Default)]
struct Parser {
	tokens: Vec<Token>,
}

impl Parser {
	fn new(code: String) -> Result<Self> {
		let mut tokens = Token::tokenize(code)?;
		tokens.reverse();
		Ok(Self { tokens })
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
		self.next_tuple(Self::next_ident)
	}

	fn next_assignee(&mut self) -> Result<ParamValue> {
		let sign = self.assume_next(Token::Invert);
		let target = self.next_ident()?;
		Ok(ParamValue { sign, target })
	}

	fn next_assignee_list(&mut self) -> Result<Vec<ParamValue>> {
		self.next_tuple(Self::next_assignee)
	}

	fn next_tuple<T>(&mut self, get_item: fn(&mut Self) -> Result<T>) -> Result<Vec<T>> {
		let items = if self.assume_next(Token::ParenthesisLeft) {
			if self.assume_next(Token::ParenthesisRight) {
				vec![]
			} else {
				let mut items: Vec<T> = vec![];
				let mut expect_item = true;

				loop {
					if expect_item {
						items.push(get_item(self)?);
					} else if !self.assume_next(Token::Comma) {
						break;
					}

					expect_item = !expect_item;
				}

				self.expect_next(Token::ParenthesisRight)?;
				items
			}
		} else {
			vec![get_item(self)?]
		};

		Ok(items)
	}
}
