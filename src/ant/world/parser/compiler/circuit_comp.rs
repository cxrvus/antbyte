use std::collections::HashMap;

use anyhow::{Result, anyhow};

use super::{FlatStatement, NormCircuit};

use crate::{
	ant::{
		compiler::{NormStatement, Normalizer},
		world::parser::{CircuitType, ParsedCircuit},
	},
	util::find_dupe,
};

pub(super) fn flatten_circuits(
	parsed_circuits: Vec<ParsedCircuit>,
) -> Result<HashMap<String, NormCircuit>> {
	let mut normalizer = Normalizer::default();

	for circuit in parsed_circuits.into_iter() {
		validate_circuit_io(&circuit)?;

		let circuit_name = circuit.name.clone();
		let flat_circuit = normalizer.flatten_circuit(circuit)?;

		if normalizer
			.0
			.insert(circuit_name.clone(), flat_circuit)
			.is_some()
		{
			return Err(anyhow!("circuit name '{circuit_name}' used more than once"));
		}
	}

	Ok(normalizer.0)
}

impl Normalizer {
	fn flatten_circuit(&self, circuit: ParsedCircuit) -> Result<NormCircuit> {
		let mut exp_index = 0;
		let mut func_index = 0;
		let mut norm_statements: Vec<NormStatement> = vec![];

		for statement in circuit.statements.iter() {
			exp_index += 1;

			let mut flat_statements = statement.flatten(&mut exp_index);

			resolve_and_gates(&mut flat_statements);

			self.validate_statements(&flat_statements, &circuit)?;

			for flat_statement in flat_statements {
				match flat_statement.call.as_str() {
					"or" => {
						if flat_statement.assignees.len() != 1 {
							return Err(anyhow!(
								"the result of an OR may only be assigned to a single assignee"
							));
						}

						norm_statements.push(flat_statement.into());
					}
					call => {
						func_index += 1;
						let expanded = self.expand_func_call(call, &flat_statement, func_index)?;
						norm_statements.extend(expanded);
					}
				}
			}

			println!("\n\n\n") //TODO: remove (dbg)
		}

		let all_assignees: Vec<_> = norm_statements.iter().map(|stm| &stm.assignee).collect();

		if let Some(dupe_assignee) = find_dupe(&all_assignees) {
			return Err(anyhow!(
				"identifier '{dupe_assignee}' can not be assigned to more than once"
			));
		}

		dbg!(&norm_statements);

		Ok(NormCircuit {
			norm_statements,
			original: circuit,
		})
	}

	fn expand_func_call(
		&self,
		call: &str,
		flat_statement: &FlatStatement,
		func_index: u32,
	) -> Result<Vec<NormStatement>> {
		let func = self
			.0
			.get(call)
			.ok_or(anyhow!("unknown function: '{call}'"))?;

		if let CircuitType::Ant(ant_type) = &func.original.circuit_type {
			return Err(anyhow!(
				"circuit '{call}' is a {ant_type:?}, not a function"
			));
		}

		validate_call_signature(func, flat_statement)?;

		let var_prefix = format!("_{call}{func_index:02}");

		let mut expanded_statements = vec![];

		for mut norm_statement in func.norm_statements.clone() {
			if let Some(out_param_index) = func
				.original
				.out_params
				.iter()
				.position(|out_param| *out_param == norm_statement.assignee)
			{
				// assignee represents a function out-param
				norm_statement.assignee = flat_statement.assignees[out_param_index].clone();
			} else {
				// assignee represents a variable
				norm_statement.assignee = var_prefix.clone() + &norm_statement.assignee;
			}

			for param in norm_statement.params.iter_mut() {
				if let Some(in_param_index) = func
					.original
					.in_params
					.iter()
					.position(|in_param| *in_param == param.target)
				{
					// value targets a function in-parameter
					let in_param_value = &flat_statement.params[in_param_index];
					param.target = in_param_value.target.clone();
					param.sign ^= in_param_value.sign;
				} else {
					// value targets a variable
					param.target = var_prefix.clone() + &param.target;
				}
			}

			expanded_statements.push(norm_statement);
		}

		Ok(expanded_statements)
	}
}

fn validate_circuit_io(circuit: &ParsedCircuit) -> Result<()> {
	if let Some(dupe_ident) = circuit.in_params.iter().find(|in_param| {
		circuit
			.out_params
			.iter()
			.any(|out_param| out_param == *in_param)
	}) {
		Err(anyhow!(
			"identifier '{dupe_ident}' used both as an in- and an out-parameter"
		))
	} else {
		Ok(())
	}
}

fn validate_call_signature(func: &NormCircuit, statement: &FlatStatement) -> Result<()> {
	let (in_count, out_count, param_val_count, assignee_count) = (
		func.original.in_params.len(),
		func.original.out_params.len(),
		statement.params.len(),
		statement.assignees.len(),
	);

	let func_name = &func.original.name;

	if param_val_count != in_count {
		Err(anyhow!(
			"function '{func_name}' has been given an invalid number of parameter values\nexpected {in_count}, got {param_val_count}"
		))
	} else if assignee_count != out_count {
		Err(anyhow!(
			"function '{func_name}' has been given an invalid number of assignees\nexpected {out_count}, got {assignee_count}"
		))
	} else {
		Ok(())
	}
}

/// transform AND into OR ([DeMorgan's Laws](https://en.wikipedia.org/wiki/De_Morgan%27s_laws))
fn resolve_and_gates(statements: &mut [FlatStatement]) {
	statements
		.iter_mut()
		.filter(|stm| stm.call == "and")
		.for_each(|stm| {
			stm.params
				.iter_mut()
				.for_each(|param| param.sign = !param.sign);

			stm.sign = !stm.sign;
			stm.call = "or".into();
		});
}
