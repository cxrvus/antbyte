pub mod compiler;

mod circuit_parser;
mod expression_parser;
mod token;
mod world_parser;

use self::token::Token;
use crate::ant::AntType;
use anyhow::{Error, Result, anyhow};

#[derive(Debug)]
enum Statement {
	Set(String, Token),
	Declare(ParsedCircuit),
}

#[derive(Debug)]
struct ParsedWorld {
	statements: Vec<Statement>,
}

#[derive(Debug)]
struct ParsedCircuit {
	name: String,
	circuit_type: CircuitType,
	inputs: Vec<String>,
	outputs: Vec<String>,
	assignments: Vec<Assignment>,
}

#[rustfmt::skip]
#[derive(Debug)]
enum CircuitType { Ant(AntType), Sub }

#[derive(Debug)]
struct Assignment {
	assignees: Vec<String>,
	expression: Expression,
}

#[derive(Debug)]
struct Expression {
	ident: String,
	sign: bool,
	/// is a function if Some, else input / hidden layer neuron
	parameters: Option<Vec<Self>>,
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
