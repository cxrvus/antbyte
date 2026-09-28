use anyhow::{Result, anyhow};
use regex::Regex;
use std::sync::LazyLock;

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

	// ## Other
	Invalid(String),
	Comment,

	#[default]
	EndOfFile,
}

const COMMENT_PTN: &str = r"#.*(?:\r?\n|$)";
const NUMBER_PTN: &str = r"(?:0[b][01]+|0[o][0-7]+|0[x][0-9a-f]+|0\d+|[1-9]\d*)";
const STRING_PTN: &str = r#""(.*?)""#;
const IDENT_PTN: &str = r"[a-zA-Z_]\w*";
const LOWER_IDENT: &str = r"_?[a-z][a-z0-9_]*";
const UPPER_IDENT: &str = r"[A-Z][A-Z0-9_]*";
const SYMBOL_PTN: &str = r"=>|,,|[#={}(),;01]|\+|-";

const SPACE_PTN: &str = r"\s+";
const WILD_PTN: &str = r".+";

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
	let pattern = [
		COMMENT_PTN,
		STRING_PTN,
		IDENT_PTN,
		NUMBER_PTN,
		SYMBOL_PTN,
		SPACE_PTN,
		WILD_PTN,
	]
	.join("|");

	regex(&pattern)
});

static WHITESPACE_RE: LazyLock<Regex> = lazy_regex_full!(SPACE_PTN);
static COMMENT_RE: LazyLock<Regex> = lazy_regex_full!(COMMENT_PTN);
static IDENT_RE: LazyLock<Regex> = lazy_regex_full!(IDENT_PTN);
static LOWER_IDENT_RE: LazyLock<Regex> = lazy_regex_full!(LOWER_IDENT);
static UPPER_IDENT_RE: LazyLock<Regex> = lazy_regex_full!(UPPER_IDENT);
static NUMBER_RE: LazyLock<Regex> = lazy_regex_full!(NUMBER_PTN);
static STRING_RE: LazyLock<Regex> = lazy_regex_full!(STRING_PTN);

impl Token {
	pub fn tokenize(code: &str) -> Result<Vec<Self>> {
		let token_strings = TOKEN_RE.find_iter(code).collect::<Vec<_>>();

		// dbg!(&token_strings.iter().map(|x| x.as_str()).collect::<Vec<_>>());

		token_strings
			.iter()
			.map(|x| x.as_str())
			.filter(|x| !(WHITESPACE_RE.is_match(x) || COMMENT_RE.is_match(x)))
			.map(Token::from_token_str)
			.collect::<Result<Vec<_>>>()
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
		} else if IDENT_RE.is_match(token) {
			if token == "_" || UPPER_IDENT_RE.is_match(token) || LOWER_IDENT_RE.is_match(token) {
				Ok(Token::Ident(token.to_string()))
			} else {
				Err(anyhow!(
					"identifiers must be either all upper or all lower-case, found '{token}'"
				))
			}
		} else if NUMBER_RE.is_match(token) {
			Self::parse_number(token).map(Token::Number)
		} else if let Some(captures) = STRING_RE.captures(token) {
			let string = captures.get(1).unwrap().as_str().to_owned();
			Ok(Token::String(string))
		} else {
			Ok(Token::Invalid(token.to_owned()))
		}
	}

	fn parse_number(token: &str) -> Result<u32> {
		fn parse_radix(digits: &str, radix: u32) -> Result<u32> {
			u32::from_str_radix(digits, radix).map_err(|e| anyhow!(e))
		}

		match token.as_bytes() {
			[b'0', b'b' | b'B', ..] => parse_radix(&token[2..], 2),
			[b'0', b'o' | b'O', ..] => parse_radix(&token[2..], 8),
			[b'0', b'x' | b'X', ..] => parse_radix(&token[2..], 16),
			_ => token.parse::<u32>().map_err(|e| anyhow!(e)),
		}
	}

	pub(super) fn is_uppercase_ident(ident: &str) -> bool {
		UPPER_IDENT_RE.is_match(ident)
	}
}
