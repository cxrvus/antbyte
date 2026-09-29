use crate::{parser::token::Token, util::bits_from_int};
use anyhow::Result;

pub fn preprocess(tokens: &Vec<Token>) -> Result<Vec<Token>> {
	let in_tokens = tokens;
	let mut out_tokens = vec![];

	for token in in_tokens {
		let mut tokens = match token {
			Token::NumberLiteral(width, value) => literal(*width, *value),
			_ => vec![token.clone()],
		};

		out_tokens.append(&mut tokens);
	}

	Ok(out_tokens)
}

fn literal(width: u8, value: u8) -> Vec<Token> {
	let mut tokens = vec![];

	let width = width as usize;
	let value = value as u16;
	let bits = &bits_from_int(value);

	for &bit in bits[bits.len() - width..].iter() {
		tokens.push(Token::Bit(bit));
		tokens.push(Token::Comma);
	}

	tokens
}
