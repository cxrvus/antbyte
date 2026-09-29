use crate::{
	parser::token::{LiteralData, RangeData, Token},
	util::bits_from_int,
};
use anyhow::{Context, Result, bail};

pub fn preprocess(tokens: &[Token]) -> Result<Vec<Token>> {
	use Token::*;

	let mut out_tokens = vec![];
	let mut in_tokens = tokens.to_vec();
	in_tokens.reverse();

	while let Some(token) = in_tokens.pop() {
		let mut tokens = match &token {
			Ident(ident) if is_void_call(out_tokens.last(), in_tokens.last()) => void_call(ident),
			NumberLiteral(data) => literal(data),
			IdentRange(data) => range(data),
			_ => Ok(vec![token.clone()]),
		}
		.context(format!("in token {token:?}"))?;

		out_tokens.append(&mut tokens);
	}

	Ok(out_tokens)
}

fn is_void_call(prev: Option<&Token>, next: Option<&Token>) -> bool {
	use Token::*;

	if let (Some(prev), Some(next)) = (prev, next) {
		matches!((prev, next), (BraceLeft | Semicolon, ParenthesisLeft))
	} else {
		false
	}
}

fn void_call(func_name: &str) -> Result<Vec<Token>> {
	use Token::*;

	Ok(vec![
		ParenthesisLeft,
		ParenthesisRight,
		Assign,
		Ident(func_name.to_string()),
	])
}

fn literal(data: &LiteralData) -> Result<Vec<Token>> {
	let LiteralData { width, value } = *data;

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

	tokens.pop(); // remove trailing comma
	Ok(tokens)
}

fn range(data: &RangeData) -> Result<Vec<Token>> {
	let RangeData {
		sign,
		ident,
		start,
		end,
	} = data.clone();

	let mut tokens = vec![];

	let range: Vec<u8> = if end > start {
		(start..=end).collect()
	} else if end < start {
		(end..=start).rev().collect()
	} else {
		bail!("range start must differ from range end")
	};

	for index in range {
		if sign {
			tokens.push(Token::Invert(true));
		}

		tokens.push(Token::parse_ident(&format!("{ident}{index}"))?);
		tokens.push(Token::Comma);
	}

	tokens.pop(); // remove trailing comma
	Ok(tokens)
}
