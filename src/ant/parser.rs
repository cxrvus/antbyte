use crate::circuit::{Circuit, Layer, WireArray};
use anyhow::{Result, anyhow};
use regex::Regex;

pub struct ParsedWorld;

pub struct Parser;

impl Parser {
	pub fn parse(code: String) -> Result<ParsedWorld> {
		let tokens = Self::tokenize(code);

		dbg!(tokens);
		todo!()
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
			.collect::<Vec<_>>()
	}
}

#[derive(Debug)]
pub enum Token {
	Invalid(String),
	Ident(String),
	Comment,
	Equals,
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
			"#" => Token::Comment,
			"=" => Token::Equals,
			"{" => Token::BraceLeft,
			"}" => Token::BraceRight,
			"(" => Token::ParenthesisLeft,
			")" => Token::ParenthesisRight,
			"," => Token::Comma,
			";" => Token::Semicolon,
			"1" => Token::True,
			"-" => Token::Invert,
			ident if Regex::new(Self::IDENT).unwrap().is_match(ident) => {
				Token::Ident(ident.to_string())
			}
			other => Token::Invalid(other.to_string()),
		}
	}
}
