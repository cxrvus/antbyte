use crate::ant::archetype::AntType;
use anyhow::{Error, Ok, Result, anyhow};
use regex::Regex;

#[derive(Debug)]
pub struct ParsedWorld {
	settings: Vec<Setting>,
	circuits: Vec<ParsedCircuit>,
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
	lhs: Vec<String>,
	rhs: Expression,
}

#[derive(Debug)]
struct Expression {
	ident: String,
	invert: bool,
	/// is a function if Some, else input / hidden layer neuron
	parameters: Option<Vec<Expression>>,
}

#[rustfmt::skip]
#[derive(Debug)]
struct Setting { key: String, value: Value }

#[rustfmt::skip]
#[derive(Debug)]
enum Value { Ident(String), Int(u32) }

#[derive(Default)]
pub struct Parser {
	tokens: Vec<Token>,
}

type Target = ParsedWorld;

#[rustfmt::skip]
enum Assumption { Correct, Incorrect(Token) }

impl Parser {
	pub fn parse(code: String) -> Result<Target> {
		let mut tokens = Self::tokenize(code);
		tokens.reverse();
		let mut parser = Self { tokens };

		parser.parse_mut()
	}

	fn parse_mut(&mut self) -> Result<Target> {
		let mut circuits: Vec<ParsedCircuit> = vec![];
		// let mut settings: Vec<Setting> = vec![] // <-- todo

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
				todo!("set world config field");
			} else if let Some(circuit_type) = match statement.as_str() {
				"queen" => Some(CircuitType::Ant(AntType::Queen)),
				"worker" => Some(CircuitType::Ant(AntType::Worker)),
				"fn" => Some(CircuitType::Sub),
				_ => None,
			} {
				let circuit = self.parse_circuit(ident, circuit_type)?;
				circuits.push(circuit);
			} else {
				return Err(anyhow!("invalid statement: {statement}"));
			}
		}

		let world = ParsedWorld {
			settings: vec![],
			circuits,
		};

		Ok(dbg!(world))
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
			inputs,
			outputs,
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

	// todo: write tests
	fn tokenize(code: String) -> Vec<Token> {
		let pattern = format!(r"{}|{}|\s+|.+", Token::SYMBOL_PTN, Token::IDENT_PTN);
		let whitespace_re = Regex::new(r"\s+").unwrap();

		Regex::new(&pattern)
			.unwrap()
			.find_iter(&code)
			.map(|x| x.as_str())
			.filter(|x| !whitespace_re.is_match(x))
			.map(Token::from)
			// .chain([Token::EndOfFile])
			.collect::<Vec<_>>()
	}
}

// idea: add Token.line and show in error handling
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Token {
	// ## Expressions
	Ident(String),
	Invert,
	ParenthesisLeft,
	ParenthesisRight,
	Comma,

	// ## Assignments / Circuits
	Semicolon,
	Assign,
	Arrow,
	BraceLeft,
	BraceRight,

	// ## Other
	Invalid(String),
	// todo: implement comments & add string value
	Comment,
	#[default]
	EndOfFile,
}

impl Token {
	pub const IDENT_PTN: &'static str = r"(\d{1,3}|([a-zA-Z]\w*))";
	pub const SYMBOL_PTN: &'static str = r"=>|[#={}(),;1]|-";
}

impl From<&str> for Token {
	fn from(value: &str) -> Self {
		match value {
			"=>" => Token::Arrow,
			"#" => Token::Comment,
			"=" => Token::Assign,
			"{" => Token::BraceLeft,
			"}" => Token::BraceRight,
			"(" => Token::ParenthesisLeft,
			")" => Token::ParenthesisRight,
			"," => Token::Comma,
			";" => Token::Semicolon,
			"-" => Token::Invert,
			ident if Regex::new(Self::IDENT_PTN).unwrap().is_match(ident) => {
				Token::Ident(ident.to_string())
			}
			other => Token::Invalid(other.to_string()),
		}
	}
}
