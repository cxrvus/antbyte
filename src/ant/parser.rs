use crate::circuit::{Circuit, Layer, WireArray};
use anyhow::{Result, anyhow};
use regex::Regex;

pub struct ParsedWorld;

pub struct Parser;

impl Parser {
	const PATTERN: &'static str = r"=>|[a-zA-Z]\w*|[#={}(),;1]|-|\s+|.+";

	pub fn parse(code: String) -> Result<ParsedWorld> {
		let tokens = Self::tokenize(code);

		dbg!(tokens);
		todo!()
	}

	fn tokenize(code: String) -> Vec<Token> {
		let whitespace_re = Regex::new(r"\s+").unwrap();

		Regex::new(Self::PATTERN)
			.unwrap()
			.find_iter(&code)
			.map(|x| x.as_str())
			.filter(|x| !whitespace_re.is_match(x))
			.map(Token::from)
			.collect::<Vec<_>>()
	}
}

#[derive(Debug)]
pub enum Token {
	Invalid(String),
	Ident(String),
}

impl From<&str> for Token {
	fn from(value: &str) -> Self {
		// idea: optimize - use string slices instead of strings

		// TODO
		Token::Ident(value.to_string())
	}
}
