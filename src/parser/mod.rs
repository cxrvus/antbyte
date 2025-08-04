use crate::{ant::archetype::AntType, parser::token::Token};
use anyhow::{Error, Result, anyhow};

mod circuit_parser;
mod expression_parser;
mod world_parser;

pub mod compiler;
pub mod token;

#[derive(Debug)]
pub enum Statement {
	Set(String, Token),
	Declare(String, ParsedCircuit),
}

#[derive(Debug)]
pub struct ParsedWorld {
	pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct ParsedCircuit {
	pub circuit_type: CircuitType,
	pub used_inputs: Vec<String>,
	pub used_outputs: Vec<String>,
	pub assignments: Vec<Assignment>,
}

#[rustfmt::skip]
#[derive(Debug)]
pub enum CircuitType { Ant(AntType), Sub }

#[derive(Debug)]
pub struct Assignment {
	pub lhs: Vec<String>,
	pub rhs: Expression,
}

#[derive(Debug)]
pub struct Expression {
	pub ident: String,
	pub sign: bool,
	/// is a function if Some, else input / hidden layer neuron
	pub parameters: Option<Vec<Self>>,
}

#[derive(Default)]
pub struct Parser {
	tokens: Vec<Token>,
}

impl Parser {
	pub fn new(code: String) -> Self {
		let mut tokens = Token::tokenize(code);
		tokens.reverse();
		Self { tokens }
	}

	pub fn parse_world(&mut self) -> Result<ParsedWorld> {
		world_parser::parse_world(self)
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
		let mut identifiers: Vec<String> = vec![];
		let mut expect_ident = true;

		loop {
			// let token = self.next_token();

			if expect_ident {
				let ident = self.next_ident()?;
				identifiers.push(ident);
			} else if !self.assume_next(Token::Comma) {
				return Ok(identifiers);
			}

			expect_ident = !expect_ident;
		}
	}
}
