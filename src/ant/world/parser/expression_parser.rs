use super::{Expression, Parser, Token};
use anyhow::{Result, anyhow};

impl Parser {
	pub(super) fn parse_next_exp(&mut self) -> Result<Expression> {
		let mut sign = false;
		let mut current_token = Token::ParenthesisLeft;
		let mut expression_sets: Vec<Vec<Expression>> = vec![];

		loop {
			let next_token = self.next_token();

			if !validate_exp_token(&current_token, &next_token) {
				let expected = expected_exp_tokens(&current_token);
				let expected_msg = format!("either: {expected:?}");
				return Err(Parser::unexpected(next_token, &expected_msg));
			}

			match &next_token {
				Token::Ident(ident) => {
					let new_exp = Expression {
						ident: ident.clone(),
						sign,
						parameters: None,
					};

					if let Some(current_set) = expression_sets.last_mut() {
						current_set.push(new_exp);
					} else {
						expression_sets.push(vec![new_exp]);
					}

					sign = false; // reset
				}

				Token::Invert => {
					sign = true;
				}

				Token::ParenthesisLeft => {
					expression_sets.push(vec![]);
				}

				Token::ParenthesisRight => {
					// if this panics, there might be an error in validate_exp_tokens()

					let parameters = expression_sets.pop().unwrap();

					if let Some(prev_set) = expression_sets.last_mut() {
						let func = prev_set.last_mut().unwrap();
						func.parameters = Some(parameters);
					} else {
						return Err(anyhow!("unmatched right parentheses"));
					}
				}

				Token::Comma => {}

				semicolon @ Token::Semicolon => {
					// re-add semicolon for assignment parsing
					self.tokens.push(semicolon.clone());

					return if expression_sets.len() == 1 {
						Ok(expression_sets.pop().unwrap().pop().unwrap())
					} else {
						Err(anyhow!(
							"unmatched left parentheses (depth = {})",
							expression_sets.len()
						))
					};
				}

				other => {
					panic!("impossible token in expression: {other:?}, after {current_token:?}")
				}
			};

			current_token = next_token;
		}
	}
}

fn validate_exp_token(current: &Token, next: &Token) -> bool {
	use Token::*;

	matches!(
		(current, next),
		(
			Ident(_),
			ParenthesisLeft | ParenthesisRight | Comma | Semicolon
		) | (Invert, Ident(_))
			| (ParenthesisLeft, Ident(_) | Invert | ParenthesisRight)
			| (ParenthesisRight, ParenthesisRight | Comma | Semicolon)
			| (Comma, Ident(_) | Invert)
	)
}

fn expected_exp_tokens(current: &Token) -> Vec<Token> {
	use Token::*;

	[
		Ident("_".into()),
		Invert,
		ParenthesisLeft,
		ParenthesisRight,
		Comma,
		Semicolon,
	]
	.into_iter()
	.filter(|next| validate_exp_token(current, next))
	.collect()
}
