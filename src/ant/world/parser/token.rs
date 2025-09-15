use anyhow::{Result, anyhow};
use regex::Regex;

#[inline]
fn regex(ptn: &str) -> Regex {
	Regex::new(ptn).unwrap()
}

#[inline]
fn regex_full(ptn: &str) -> Regex {
	regex(&format!("^{ptn}$"))
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

impl Token {
	const COMMENT_PTN: &'static str = r"#.*(?:\r?\n|$)";
	const NUMBER_PTN: &'static str = r"\d{1,3}"; // todo: parse bin and hex numbers
	const STRING_PTN: &'static str = r#""(.*?)""#;
	const IDENT_PTN: &'static str = r"[a-zA-Z_]\w*";
	const LOWER_IDENT: &'static str = r"[a-z][a-z0-9_]*";
	const UPPER_IDENT: &'static str = r"[A-Z][A-Z0-9_]*";
	const SYMBOL_PTN: &'static str = r"=>|[#={}(),;01]|\+|-";

	const SPACE_PTN: &'static str = r"\s+";
	const WILD_PTN: &'static str = r".+";

	pub fn tokenize(code: &str) -> Result<Vec<Self>> {
		let pattern = [
			Self::COMMENT_PTN,
			Self::STRING_PTN,
			Self::IDENT_PTN,
			Self::NUMBER_PTN,
			Self::SYMBOL_PTN,
			Self::SPACE_PTN,
			Self::WILD_PTN,
		]
		.join("|");

		let token_strings = regex(&pattern).find_iter(code).collect::<Vec<_>>();

		let whitespace_re = regex_full(Self::SPACE_PTN);
		let comment_re = regex_full(Self::COMMENT_PTN);

		// dbg!(&token_strings.iter().map(|x| x.as_str()).collect::<Vec<_>>());

		token_strings
			.iter()
			.map(|x| x.as_str())
			.filter(|x| !(whitespace_re.is_match(x) || comment_re.is_match(x)))
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
			"=>" => Some(Token::Arrow),
			"#" => Some(Token::Comment),
			"=" => Some(Token::Assign),
			"{" => Some(Token::BraceLeft),
			"}" => Some(Token::BraceRight),
			"(" => Some(Token::ParenthesisLeft),
			")" => Some(Token::ParenthesisRight),
			"," => Some(Token::Comma),
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
		} else if regex_full(Self::IDENT_PTN).is_match(token) {
			if token == "_"
				|| regex_full(Self::UPPER_IDENT).is_match(token)
				|| regex_full(Self::LOWER_IDENT).is_match(token)
			{
				Ok(Token::Ident(token.to_string()))
			} else {
				Err(anyhow!(
					"identifiers must be either all upper or all lower-case, found '{token}'"
				))
			}
		} else if regex_full(Self::NUMBER_PTN).is_match(token) {
			token
				.parse::<u32>()
				.map(Token::Number)
				.map_err(|e| anyhow!(e))
		} else if let Some(captures) = regex_full(Self::STRING_PTN).captures(token) {
			let string = captures.get(1).unwrap().as_str().to_owned();
			Ok(Token::String(string))
		} else {
			Ok(Token::Invalid(token.to_owned()))
		}
	}

	pub(super) fn is_uppercase_ident(ident: &str) -> bool {
		regex_full(Self::UPPER_IDENT).is_match(ident)
	}
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Keyword { Set, Fn, Ant }

impl Keyword {
	pub(super) fn from_ident(ident: &str) -> Option<Self> {
		match ident {
			"set" => Some(Self::Set),
			"fn" => Some(Self::Fn),
			"ant" => Some(Self::Ant),
			_ => None,
		}
	}
}
