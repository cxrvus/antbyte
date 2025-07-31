use crate::{ant::archetype::AntType, parser::token::Token};
use anyhow::{Error, Result, anyhow};

#[derive(Debug)]
pub enum Statement {
	Set(Setting),
	Declare(ParsedCircuit),
}

#[derive(Debug)]
pub struct ParsedWorld {
	pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct ParsedCircuit {
	pub name: String,
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
	lhs: Vec<String>,
	rhs: Expression,
}

#[derive(Debug)]
pub struct Expression {
	ident: String,
	invert: bool,
	/// is a function if Some, else input / hidden layer neuron
	parameters: Option<Vec<Self>>,
}

#[rustfmt::skip]
#[derive(Debug)]
pub struct Setting { pub key: String, pub value: Token }

// idea: remove Assumption
#[rustfmt::skip]
enum Assumption { Correct, Incorrect(Token) }

#[derive(Default)]
pub struct Lexer {
	tokens: Vec<Token>,
}

type Target = ParsedWorld;

impl Lexer {
	pub fn parse(code: String) -> Result<Target> {
		let mut tokens = Token::tokenize(code);
		tokens.reverse();
		let mut parser = Self { tokens };

		parser.parse_world()
	}

	fn parse_world(&mut self) -> Result<Target> {
		let mut statements: Vec<Statement> = vec![];

		loop {
			let statement = match self.next_token() {
				Token::Ident(ident) => ident,
				Token::EndOfFile => break,
				// fixme: better error handling - parsing goes on even if statement is invalid
				other => return Err(Self::unexpected(other, "statement")),
			};

			let ident = self.next_ident()?;

			match self.next_token() {
				Token::Assign => {}
				other => return Err(Self::unexpected(other, "'='")),
			};

			if statement.as_str() == "set" {
				let setting = self.parse_setting(ident)?;
				statements.push(Statement::Set(setting));
			} else if let Some(circuit_type) = match statement.as_str() {
				"queen" => Some(CircuitType::Ant(AntType::Queen)),
				"worker" => Some(CircuitType::Ant(AntType::Worker)),
				"fn" => Some(CircuitType::Sub),
				_ => None,
			} {
				let circuit = self.parse_circuit(ident, circuit_type)?;
				statements.push(Statement::Declare(circuit));
			} else {
				return Err(anyhow!("invalid statement: {statement}"));
			}
		}

		let world = ParsedWorld { statements };

		Ok(dbg!(world))
	}

	fn parse_setting(&mut self, key: String) -> Result<Setting> {
		self.assume_next(Token::Assign);
		let value = self.next_token();
		self.expect_next(Token::Semicolon)?;

		match value {
			Token::Ident(_) | Token::Number(_) => Ok(Setting { key, value }),
			other => Err(Self::unexpected(other, "value (identifier or number)")),
		}
	}

	fn parse_circuit(&mut self, name: String, circuit_type: CircuitType) -> Result<ParsedCircuit> {
		let inputs = self.next_ident_list()?;

		self.expect_next(Token::Arrow)?;

		let outputs: Vec<String> = self.next_ident_list()?;

		self.expect_next(Token::BraceLeft)?;

		let mut assignments: Vec<Assignment> = vec![];

		loop {
			let lhs = self.next_ident_list()?;

			self.expect_next(Token::Assign)?;

			let rhs = self.next_expression()?;
			assignments.push(Assignment { lhs, rhs });

			self.expect_next(Token::Semicolon)?;

			if let Assumption::Correct = self.assume_next(Token::BraceRight) {
				break;
			}
		}

		let circuit = ParsedCircuit {
			name,
			circuit_type,
			used_inputs: inputs,
			used_outputs: outputs,
			assignments,
		};

		Ok(circuit)
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

	fn expect_next(&mut self, expected: Token) -> Result<()> {
		Self::expect(self.next_token(), expected)
	}

	fn assume_next(&mut self, expected: Token) -> Assumption {
		let actual = self.next_token();
		if actual == expected {
			Assumption::Correct
		} else {
			self.tokens.push(actual.clone());
			Assumption::Incorrect(actual)
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
			} else if let Assumption::Incorrect(_) = self.assume_next(Token::Comma) {
				return Ok(identifiers);
			}

			expect_ident = !expect_ident;
		}
	}

	fn next_expression(&mut self) -> Result<Expression> {
		let mut invert = false;
		let mut current_token = Token::ParenthesisLeft;
		let mut expression_sets: Vec<Vec<Expression>> = vec![];

		loop {
			let next_token = self.next_token();

			if !Self::validate_exp_token(&current_token, &next_token) {
				let expected = Self::expected_exp_tokens(&current_token);
				let expected_msg = format!("either: {expected:?}");
				return Err(Self::unexpected(current_token, &expected_msg));
			}

			match &next_token {
				Token::Ident(ident) => {
					let new_exp = Expression {
						ident: ident.clone(),
						invert,
						parameters: None,
					};

					if let Some(current_set) = expression_sets.last_mut() {
						current_set.push(new_exp);
					} else {
						expression_sets.push(vec![new_exp]);
					}

					invert = false; // reset
				}

				Token::Invert => {
					invert = true;
				}

				Token::ParenthesisLeft => {
					expression_sets.push(vec![]);
				}

				Token::ParenthesisRight => {
					// if this panics, there might be an error in validate_exp_tokens()

					let parameters = expression_sets.pop().unwrap();

					if let Some(prev_set) = expression_sets.last_mut() {
						let func = prev_set.last_mut().unwrap();
						func.parameters = Some(parameters);
					} else {
						return Err(anyhow!("unmatched right parentheses"));
					}
				}

				Token::Comma => {}

				semicolon @ Token::Semicolon => {
					// re-add semicolon for assignment parsing
					self.tokens.push(semicolon.clone());

					return if expression_sets.len() == 1 {
						Ok(expression_sets.pop().unwrap().pop().unwrap())
					} else {
						Err(anyhow!(
							"unmatched left parentheses (depth = {})",
							expression_sets.len()
						))
					};
				}

				other => {
					panic!("impossible token in expression: {other:?}, after {current_token:?}")
				}
			};

			current_token = next_token;
		}
	}

	fn validate_exp_token(current: &Token, next: &Token) -> bool {
		use Token::*;

		matches!(
			(current, next),
			(
				Ident(_),
				ParenthesisLeft | ParenthesisRight | Comma | Semicolon
			) | (Invert, Ident(_))
				| (ParenthesisLeft, Ident(_) | Invert | ParenthesisRight)
				| (ParenthesisRight, ParenthesisRight | Comma | Semicolon)
				| (Comma, Ident(_) | Invert)
		)
	}

	fn expected_exp_tokens(current: &Token) -> Vec<Token> {
		use Token::*;

		[
			Ident("_".into()),
			Invert,
			ParenthesisLeft,
			ParenthesisRight,
			Comma,
			Semicolon,
		]
		.into_iter()
		.filter(|next| Self::validate_exp_token(current, next))
		.collect()
	}
}
