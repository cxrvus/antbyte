use crate::{
	ant::world::parser::{AntFunc, Func, Signature},
	util::find_dupe,
};

use super::{Parser, Statement, Token};

use anyhow::{Result, anyhow};

const MAIN: &str = "main";

impl Parser {
	pub(super) fn parse_ant(&mut self, name: String) -> Result<(Func, AntFunc)> {
		let target_id = if self.assume_next(Token::Assign) {
			let target_id = self.next_token();
			if let Token::Number(target_id) = target_id {
				target_id as u8
			} else {
				return Err(anyhow!("expected Ant target ID after '='"));
			}
		} else if name == MAIN {
			1
		} else {
			return Err(anyhow!("specify Ant target ID using '='"));
		};

		let ant = AntFunc {
			target_name: name.clone(),
			target_id,
		};

		let statements = self.parse_statements()?;

		let func = Func {
			statements,
			signature: Signature {
				name,
				..Default::default()
			},
		};

		Ok((func, ant))
	}

	pub(super) fn parse_func(&mut self, name: String) -> Result<Func> {
		self.expect_next(Token::Assign)?;

		let signature = self.parse_signature(name)?;
		let statements = self.parse_statements()?;

		Ok(Func {
			statements,
			signature,
		})
	}

	fn parse_signature(&mut self, name: String) -> Result<Signature> {
		let params = self.next_ident_list()?;
		self.expect_next(Token::Arrow)?;
		let assignees: Vec<String> = self.next_ident_list()?;

		let signature = Signature {
			name,
			params,
			assignees,
		};

		signature.validate()?;

		Ok(signature)
	}

	fn parse_statements(&mut self) -> Result<Vec<Statement>> {
		self.expect_next(Token::BraceLeft)?;

		let mut statements: Vec<Statement> = vec![];

		while !self.assume_next(Token::BraceRight) {
			let assignees = self.next_assignee_list()?;

			self.expect_next(Token::Assign)?;

			let expression = self.parse_next_exp()?;

			statements.push(Statement {
				assignees,
				expression,
			});

			self.expect_next(Token::Semicolon)?;
		}

		Ok(statements)
	}
}

impl Signature {
	fn validate(&self) -> Result<()> {
		self.validate_keywords()?;

		if let Some(collision) = self
			.params
			.iter()
			.find(|param| self.assignees.iter().any(|assignee| assignee == *param))
		{
			Err(anyhow!(
				"identifier '{collision}' used both as a parameter and an assignee"
			))
		} else if self.params.contains(&self.name) || self.assignees.contains(&self.name) {
			Err(anyhow!(
				"cannot use func name {} as parameter or assignee",
				self.name
			))
		} else if let Some(dupe) = find_dupe(&self.params) {
			Err(anyhow!("identifier {dupe} used for multiple parameters"))
		} else if let Some(dupe) = find_dupe(&self.assignees) {
			Err(anyhow!("identifier {dupe} used for multiple assignees"))
		} else {
			Ok(())
		}
	}

	fn validate_keywords(&self) -> Result<()> {
		let Signature {
			name,
			assignees,
			params,
		} = self;

		let mut idents = vec![name];
		idents.extend(params);
		idents.extend(assignees);

		for ident in idents {
			if Token::is_uppercase_ident(ident) {
				return Err(anyhow!(
					"may only use lower-case identifiers in function signatures\nfound '{ident}' in function '{name}'"
				));
			}
		}

		Ok(())
	}
}
