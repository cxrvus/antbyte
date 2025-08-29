use anyhow::{Result, anyhow};
use regex::Regex;

#[inline]
fn regex(ptn: &str) -> Regex {
	Regex::new(ptn).unwrap()
}

// idea: add Token.line and show in error handling
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Token {
	Ident(String),

	// ## Expressions
	// todo: let it be Invert(bool), so that a *buffer* ("+") is possible
	Invert,
	ParenthesisLeft,
	ParenthesisRight,
	Comma,
	BraceLeft,
	BraceRight,
	// todo: implement constant ONE and ZERO

	// ## Top-Level / Funcs
	Keyword(Keyword),
	Semicolon,
	Assign,
	Arrow,

	// ## Values
	String(String), // todo: implement
	Number(u32),

	// ## Other
	Invalid(String),
	Comment, // todo: implement

	#[default]
	EndOfFile,
}

impl Token {
	// idea: allow hex numbers
	const NUMBER_PTN: &'static str = r"\d{1,3}";
	const IDENT_PTN: &'static str = r"[a-zA-Z_]\w*";
	const LOWER_IDENT: &'static str = r"^[a-z][a-z0-9_]*$";
	const UPPER_IDENT: &'static str = r"^[A-Z][A-Z0-9_]*$";
	const SYMBOL_PTN: &'static str = r"=>|[#={}(),;1]|-";

	const SPACE_PTN: &'static str = r"\s+";
	const WILD_PTN: &'static str = r".+";

	// todo: write tests
	pub fn tokenize(code: String) -> Result<Vec<Self>> {
		let pattern = [
			Self::IDENT_PTN,
			Self::NUMBER_PTN,
			Self::SYMBOL_PTN,
			Self::SPACE_PTN,
			Self::WILD_PTN,
		]
		.join("|");

		let whitespace_re = regex(Self::SPACE_PTN);

		regex(&pattern)
			.find_iter(&code)
			.map(|x| x.as_str())
			.filter(|x| !whitespace_re.is_match(x))
			.map(Token::from_token_str)
			// .chain([Token::EndOfFile])
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
			"-" => Some(Token::Invert),
			_ => None,
		}
	}

	fn complex_match(token: &str) -> Result<Self> {
		if let Some(keyword) = Keyword::from_ident(token) {
			Ok(Token::Keyword(keyword))
		} else if regex(Self::IDENT_PTN).is_match(token) {
			if regex(Self::UPPER_IDENT).is_match(token) || regex(Self::LOWER_IDENT).is_match(token)
			{
				Ok(Token::Ident(token.to_string()))
			} else {
				Err(anyhow!(
					"identifiers must be either all upper or all lower-case, found '{token}'"
				))
			}
		} else if regex(Self::NUMBER_PTN).is_match(token) {
			token
				.parse::<u32>()
				.map(Token::Number)
				.map_err(|e| anyhow!(e))
		} else {
			Ok(Token::Invalid(token.to_owned()))
		}
	}

	pub(super) fn is_uppercase_ident(ident: &str) -> bool {
		regex(Self::UPPER_IDENT).is_match(ident)
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
