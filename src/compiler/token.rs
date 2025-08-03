use regex::Regex;

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

	// ## Assignments / Circuits
	Semicolon,
	Assign,
	Arrow,
	BraceLeft,
	BraceRight,

	// ## Values
	// todo: implement
	String(String),
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
	const SYMBOL_PTN: &'static str = r"=>|[#={}(),;1]|-";

	const SPACE_PTN: &'static str = r"\s+";
	const WILD_PTN: &'static str = r".+";

	// todo: write tests
	pub fn tokenize(code: String) -> Vec<Self> {
		let pattern = [
			Self::NUMBER_PTN,
			Self::IDENT_PTN,
			Self::SYMBOL_PTN,
			Self::SPACE_PTN,
			Self::WILD_PTN,
		]
		.join("|");

		let whitespace_re = Regex::new(Self::SPACE_PTN).unwrap();

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
			number if Regex::new(Self::NUMBER_PTN).unwrap().is_match(number) => {
				let number = number.parse::<u32>();
				match number {
					Ok(number) => Token::Number(number),
					Err(error) => Token::Invalid(error.to_string()),
				}
			}
			other => Token::Invalid(other.to_string()),
		}
	}
}
