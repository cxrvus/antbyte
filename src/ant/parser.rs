use crate::{ant::archetype::AntType, world::WorldConfig};
use anyhow::{Error, Ok, Result, anyhow};
use regex::Regex;

#[derive(Debug)]
struct ParsedCircuit {
	name: String,
	circuit_type: CircuitType,
	inputs: Vec<String>,
	outputs: Vec<String>,
	assignments: Vec<Assignment>,
}

#[derive(Debug)]
enum CircuitType {
	Ant(AntType),
	Sub,
}

#[derive(Debug)]
struct Assignment {
	lhs: String,
	// TODO: should be Value instead of Vec<Token>
	rhs: Vec<Token>,
}

enum Value {
	Ident(String),
	Call(Call),
}

struct Call {
	function: String,
	parameters: Vec<Value>,
}

#[derive(Default)]
enum Scope {
	#[default]
	Global,
	Circuit,
}

#[derive(Default)]
pub struct Parser {
	tokens: Vec<Token>,
	world: WorldConfig,
	scope: Scope,
	circuits: Vec<ParsedCircuit>,
}

type Target = WorldConfig;

impl Parser {
	// idea: dynamically generate expected token list using a matrix
	#[inline]
	fn unexpected(unexpected: Token, expected: &str) -> Error {
		anyhow!("unexpected token: {unexpected:?}, expected {expected}")
	}

	pub fn parse(code: String) -> Result<Target> {
		let mut tokens = Self::tokenize(code);
		tokens.reverse();

		let mut parser = Self {
			tokens,
			..Default::default()
		};

		loop {
			let statement = match parser.next_token() {
				Token::Text(text) => text,
				Token::EndOfFile => break,
				other => return Err(Self::unexpected(other, "statement")),
			};

			let ident = match parser.next_token() {
				Token::Text(text) => text,
				other => return Err(Self::unexpected(other, "identifier")),
			};

			match parser.next_token() {
				Token::Assign => {}
				other => return Err(Self::unexpected(other, "'='")),
			};

			match statement.as_str() {
				"set" => todo!("set world config field"),
				"queen" => parser.parse_circuit(ident, CircuitType::Ant(AntType::Queen)),
				"worker" => parser.parse_circuit(ident, CircuitType::Ant(AntType::Worker)),
				"fn" => parser.parse_circuit(ident, CircuitType::Sub),
				other => return Err(anyhow!("invalid statement: {other}")),
			}?;
		}

		// TODO
		dbg!(parser.circuits);

		Ok(parser.world)
	}

	fn parse_circuit(&mut self, name: String, circuit_type: CircuitType) -> Result<()> {
		let inputs = self.parse_ident_list()?;

		self.expect_next(Token::Arrow)?;

		let outputs: Vec<String> = self.parse_ident_list()?;

		self.expect_next(Token::BraceLeft)?;

		let mut assignments: Vec<Assignment> = vec![];

		loop {
			let lhs = match self.next_token() {
				Token::Text(ident) => ident,
				other => return Err(Self::unexpected(other, "identifier")),
			};

			self.expect_next(Token::Assign)?;

			let mut rhs = vec![];

			loop {
				match self.next_token() {
					Token::Semicolon => break,
					// TODO: stricter
					other => rhs.push(other),
				}
			}

			assignments.push(Assignment { lhs, rhs });

			let token = self.next_token();

			if let Token::BraceRight = token {
				break;
			} else {
				self.tokens.push(token);
			}
		}

		let circuit = ParsedCircuit {
			name,
			circuit_type,
			inputs,
			outputs,
			assignments,
		};

		self.circuits.push(circuit);
		Ok(())
	}

	#[inline]
	fn next_token(&mut self) -> Token {
		self.tokens.pop().unwrap_or_default()
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

	fn parse_ident_list(&mut self) -> Result<Vec<String>> {
		let mut identifiers: Vec<String> = vec![];
		let mut expect_ident = true;

		loop {
			let token = self.next_token();

			if expect_ident {
				if let Token::Text(text) = token {
					identifiers.push(text)
				} else {
					return Err(Self::unexpected(token, "identifier"));
				}
			} else {
				if !matches!(token, Token::Comma) {
					self.tokens.push(token);
					return Ok(identifiers);
				}
			}
			expect_ident = !expect_ident;
		}
	}

	// todo: write tests
	fn tokenize(code: String) -> Vec<Token> {
		let pattern = format!(r"{}|{}|\s+|.+", Token::SYMBOL, Token::IDENT);
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Token {
	#[default]
	EndOfFile,
	Invalid(String),
	Text(String),
	Arrow,
	Comment,
	Assign,
	BraceLeft,
	BraceRight,
	ParenthesisLeft,
	ParenthesisRight,
	Comma,
	Semicolon,
	True,
	Invert,
}

impl Token {
	pub const IDENT: &'static str = r"[a-zA-Z]\w*";
	pub const SYMBOL: &'static str = r"=>|[#={}(),;1]|-";
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
			"1" => Token::True,
			"-" => Token::Invert,
			ident if Regex::new(Self::IDENT).unwrap().is_match(ident) => {
				Token::Text(ident.to_string())
			}
			other => Token::Invalid(other.to_string()),
		}
	}
}
