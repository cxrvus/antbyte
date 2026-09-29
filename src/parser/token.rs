use anyhow::{Result, anyhow, bail};
use regex::Regex;
use std::sync::LazyLock;

use crate::parser::preprocess::preprocess;

use super::Keyword;

#[inline]
fn regex(ptn: &str) -> Regex {
	Regex::new(ptn).unwrap()
}

#[inline]
fn regex_full(ptn: &str) -> Regex {
	regex(&format!("^{ptn}$"))
}

macro_rules! lazy_regex_full {
	($ptn:expr) => {
		LazyLock::new(|| regex_full($ptn))
	};
}

// idea: add Token line metadata
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Token {
	Ident(String),

	// ## Expressions
	Invert(bool),
	ParenthesisLeft,
	ParenthesisRight,
	Comma,
	BraceLeft,
	BraceRight,
	Bit(bool),

	// ## Top-Level / Funcs
	Keyword(Keyword),
	Semicolon,
	Assign,
	Arrow,

	// ## Values
	String(String),
	Number(u32),

	// ## Pre-Processor
	NumberLiteral(u8, u8),
	IdentRange(String, u8, u8),

	// ## Other
	Invalid(String),
	Comment,

	#[default]
	EndOfFile,
}

#[rustfmt::skip]
macro_rules! number_ptn { () => { r"0[b][01]+|0[o][0-7]+|0[x][0-9a-f]+|0\d+|[1-9]\d*" }; }

const COMMENT_PTN: &str = r"#.*(?:\r?\n|$)";
const RANGE_PTN: &str = r"([a-zA-Z_]\w*?)([0-7])\.\.([0-7])";
const IDENT_PTN: &str = r"[a-zA-Z_]\w*";
const LITERAL_PTN: &str = concat!(r"([1-8])'(", number_ptn!(), ")");
const NUMBER_PTN: &str = concat!(r"(?:", number_ptn!(), ")");
const SPACE_PTN: &str = r"\s+";
const STRING_PTN: &str = r#""(.*?)""#;
const SYMBOL_PTN: &str = r"=>|,,|[#={}(),;01]|\+|-";
const WILD_PTN: &str = r".+";

const LOWER_IDENT: &str = r"_?[a-z][a-z0-9_]*";
const UPPER_IDENT: &str = r"[A-Z][A-Z0-9_]*";

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
	// order matters!
	let pattern = [
		COMMENT_PTN,
		STRING_PTN,
		RANGE_PTN,
		IDENT_PTN,
		LITERAL_PTN,
		NUMBER_PTN,
		SYMBOL_PTN,
		SPACE_PTN,
		WILD_PTN,
	]
	.join("|");

	regex(&pattern)
});

static COMMENT_RE: LazyLock<Regex> = lazy_regex_full!(COMMENT_PTN);
static RANGE_RE: LazyLock<Regex> = lazy_regex_full!(RANGE_PTN);
static IDENT_RE: LazyLock<Regex> = lazy_regex_full!(IDENT_PTN);
static LITERAL_RE: LazyLock<Regex> = lazy_regex_full!(LITERAL_PTN);
static NUMBER_RE: LazyLock<Regex> = lazy_regex_full!(NUMBER_PTN);
static SPACE_RE: LazyLock<Regex> = lazy_regex_full!(SPACE_PTN);
static STRING_RE: LazyLock<Regex> = lazy_regex_full!(STRING_PTN);

static LOWER_IDENT_RE: LazyLock<Regex> = lazy_regex_full!(LOWER_IDENT);
static UPPER_IDENT_RE: LazyLock<Regex> = lazy_regex_full!(UPPER_IDENT);

impl Token {
	pub fn tokenize(code: &str) -> Result<Vec<Self>> {
		let token_strings = TOKEN_RE.find_iter(code).collect::<Vec<_>>();

		// dbg!(&token_strings.iter().map(|x| x.as_str()).collect::<Vec<_>>());

		let tokens = token_strings
			.iter()
			.map(|x| x.as_str())
			.filter(|x| !(SPACE_RE.is_match(x) || COMMENT_RE.is_match(x)))
			.map(Token::from_token_str)
			.collect::<Result<Vec<_>>>()?;

		preprocess(&tokens)
	}

	fn from_token_str(value: &str) -> Result<Self> {
		Self::simple_match(value)
			.map(Ok)
			.unwrap_or_else(|| Self::complex_match(value))
	}

	fn simple_match(token: &str) -> Option<Self> {
		match token {
			"," | ",," => Some(Token::Comma),
			"=>" => Some(Token::Arrow),
			"#" => Some(Token::Comment),
			"=" => Some(Token::Assign),
			"{" => Some(Token::BraceLeft),
			"}" => Some(Token::BraceRight),
			"(" => Some(Token::ParenthesisLeft),
			")" => Some(Token::ParenthesisRight),
			";" => Some(Token::Semicolon),
			"+" => Some(Token::Invert(false)),
			"-" => Some(Token::Invert(true)),
			"0" => Some(Token::Bit(false)),
			"1" => Some(Token::Bit(true)),
			_ => None,
		}
	}

	fn complex_match(token: &str) -> Result<Self> {
		if let Some(keyword) = Keyword::from_ident(token) {
			Ok(Token::Keyword(keyword))
		} else if RANGE_RE.is_match(token) {
			Self::parse_range(token)
		} else if IDENT_RE.is_match(token) {
			Self::parse_ident(token)
		} else if LITERAL_RE.is_match(token) {
			Self::parse_literal(token)
		} else if NUMBER_RE.is_match(token) {
			Self::parse_number(token).map(Token::Number)
		} else if let Some(captures) = STRING_RE.captures(token) {
			let string = captures.get(1).unwrap().as_str().to_owned();
			Ok(Token::String(string))
		} else {
			Ok(Token::Invalid(token.to_owned()))
		}
	}

	fn parse_range(token: &str) -> Result<Token> {
		let captures = RANGE_RE.captures(token).unwrap();
		let ident = captures.get(1).unwrap().as_str().to_string();
		let start = captures.get(2).unwrap().as_str().parse::<u8>().unwrap();
		let end__ = captures.get(3).unwrap().as_str().parse::<u8>().unwrap();

		Ok(Token::IdentRange(ident, start, end__))
	}

	pub fn parse_ident(token: &str) -> Result<Token> {
		if token == "_" || UPPER_IDENT_RE.is_match(token) || LOWER_IDENT_RE.is_match(token) {
			Ok(Token::Ident(token.to_string()))
		} else {
			Err(anyhow!(
				"identifiers must be either all upper or all lower-case, found '{token}'"
			))
		}
	}

	fn parse_literal(token: &str) -> Result<Token> {
		let captures = LITERAL_RE.captures(token).unwrap();
		let width = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();
		let value = captures.get(2).unwrap().as_str();

		let number = Self::parse_number(value)?;

		if number > u8::MAX as u32 {
			bail!("number literal may not be greater than 255, in [{token}]");
		}

		Ok(Token::NumberLiteral(width, number as u8))
	}

	fn parse_number(token: &str) -> Result<u32> {
		fn parse_radix(digits: &str, radix: u32) -> Result<u32> {
			u32::from_str_radix(digits, radix).map_err(|e| anyhow!(e))
		}

		match token.as_bytes() {
			[b'0', b'b', ..] => parse_radix(&token[2..], 2),
			[b'0', b'o', ..] => parse_radix(&token[2..], 8),
			[b'0', b'x', ..] => parse_radix(&token[2..], 16),
			_ => token.parse::<u32>().map_err(|e| anyhow!(e)),
		}
	}

	pub(super) fn is_uppercase_ident(ident: &str) -> bool {
		UPPER_IDENT_RE.is_match(ident)
	}
}
