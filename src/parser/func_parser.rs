use crate::{
	parser::{Func, Signature},
	util::find_dupe,
};

use super::{Parser, Statement, Token};

use anyhow::{Result, anyhow, bail};

impl Parser {
	pub(super) fn parse_func(&mut self, signature: Signature) -> Result<Func> {
		Ok(Func {
			statements: self.parse_statements()?,
			signature,
		})
	}

	pub(super) fn parse_signature(&mut self, name: &str) -> Result<Signature> {
		let params = self.next_ident_list()?;

		let assignees = if self.assume_next(Token::Arrow).is_some() {
			self.next_ident_list()?
		} else {
			vec![]
		};

		let signature = Signature {
			name: name.to_owned(),
			params,
			assignees,
		};

		signature.validate()?;

		Ok(signature)
	}

	fn parse_statements(&mut self) -> Result<Vec<Statement>> {
		self.expect_next(Token::BraceLeft)?;

		let mut statements: Vec<Statement> = vec![];

		while self.assume_next(Token::BraceRight).is_none() {
			let assignees = self.next_assignee_list()?;

			let assignee_targets: &Vec<_> = &assignees
				.iter()
				.map(|x| &x.target)
				.filter(|&x| x != "_")
				.collect();

			if let Some(dupe_target) = find_dupe(assignee_targets) {
				bail!("found duplicate assignee targeting '{dupe_target}' in statement");
			}

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
		if self.name == "or" {
			bail!("may not use 'or' as a function name")
		}

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
				"cannot use func name '{}' as parameter or assignee",
				self.name
			))
		} else if let Some(dupe) = find_dupe(&self.params) {
			Err(anyhow!("identifier '{dupe}' used for multiple parameters"))
		} else if let Some(dupe) = find_dupe(&self.assignees) {
			Err(anyhow!("identifier '{dupe}' used for multiple assignees"))
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
				bail!(
					"may only use lower-case identifiers in function signatures\nfound '{ident}' in function '{name}'"
				);
			}
		}

		Ok(())
	}
}
