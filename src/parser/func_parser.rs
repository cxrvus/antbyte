use std::mem::take;

use crate::{
	ant::sub_pin::SubPin,
	parser::{Expression, Func, ParamValue, ParsedWorld, Signature},
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

		Signature::new(name, &params, &assignees)
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

	pub(super) fn parse_link(&mut self, world: &mut ParsedWorld) -> Result<()> {
		let name = self.next_ident()?;
		let targets = self.next_ident_list()?;
		self.expect_next(Token::Semicolon)?;

		if targets.is_empty() {
			bail!("link must have at least one target");
		} else if let Some(err) = targets.iter().find_map(|x| SubPin::from_ident(x).err()) {
			bail!("all link targets must be valid pins.\n    {err}")
		}

		let params: Vec<String> = (0..targets.len()).map(|i| format!("x{i:02o}")).collect();

		fn assignment(assignee: &str, expression: &str) -> Statement {
			Statement {
				assignees: vec![ParamValue {
					target: assignee.to_owned(),
					sign: false,
				}],
				expression: Expression {
					ident: expression.to_owned(),
					sign: false,
					params: None,
				},
			}
		}

		let mut statements = vec![];

		// Setter...

		for (i, target) in targets.iter().enumerate() {
			statements.push(assignment(target, &params[i]));
		}

		let signature = Signature::new(&name, &params, &[])?;

		let setter = Func {
			signature,
			statements: take(&mut statements),
		};

		// Getter...

		let signature = Signature::new(&name, &[], &params)?;

		for (i, target) in targets.iter().enumerate() {
			statements.push(assignment(&params[i], target));
		}

		let getter = Func {
			signature,
			statements,
		};

		world.funcs.push(setter);
		world.funcs.push(getter);

		Ok(())
	}
}

impl Signature {
	pub fn new(name: &str, params: &[String], assignees: &[String]) -> Result<Self> {
		if name == "or" {
			bail!("may not use 'or' as a function name")
		}

		if let Some(collision) = params
			.iter()
			.find(|param| assignees.iter().any(|assignee| assignee == *param))
		{
			Err(anyhow!(
				"identifier '{collision}' used both as a parameter and an assignee"
			))
		} else if params.contains(&name.to_string()) || assignees.contains(&name.to_string()) {
			Err(anyhow!(
				"cannot use func name '{}' as parameter or assignee",
				name
			))
		} else if let Some(dupe) = find_dupe(params) {
			Err(anyhow!("identifier '{dupe}' used for multiple parameters"))
		} else if let Some(dupe) = find_dupe(assignees) {
			Err(anyhow!("identifier '{dupe}' used for multiple assignees"))
		} else {
			let signature = Self {
				name: name.to_owned(),
				params: params.into(),
				assignees: assignees.into(),
			};
			signature.validate_keywords()?;
			Ok(signature)
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
