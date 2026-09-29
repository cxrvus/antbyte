use crate::{parser::token::Token, util::bits_from_int};
use anyhow::{Context, Result, bail};

pub fn preprocess(tokens: &Vec<Token>) -> Result<Vec<Token>> {
	let in_tokens = tokens;
	let mut out_tokens = vec![];

	for token in in_tokens {
		let mut tokens = match token {
			Token::NumberLiteral(width, value) => literal(*width, *value),
			Token::IdentRange(ident, start, end) => range(ident, *start, *end),
			_ => Ok(vec![token.clone()]),
		}
		.context(format!("in token {token:?}"))?;

		out_tokens.append(&mut tokens);
	}

	Ok(out_tokens)
}

fn literal(width: u8, value: u8) -> Result<Vec<Token>> {
	if width < 8 && value >= 1 << width {
		bail!("value {value} must be less than {}", 1 << width);
	}

	let mut tokens = vec![];

	let width = width as usize;
	let value = value as u16;
	let bits = &bits_from_int(value);

	for &bit in bits[bits.len() - width..].iter() {
		tokens.push(Token::Bit(bit));
		tokens.push(Token::Comma);
	}

	Ok(tokens)
}

fn range(ident: &str, start: u8, end: u8) -> Result<Vec<Token>> {
	let mut tokens = vec![];

	let range: Vec<u8> = if end > start {
		(start..=end).collect()
	} else if end < start {
		(end..=start).rev().collect()
	} else {
		bail!("range start must differ from range end")
	};

	for index in range {
		let full_ident = format!("{ident}{index}");
		tokens.push(Token::parse_ident(&full_ident)?);
		tokens.push(Token::Comma);
	}

	Ok(tokens)
}
