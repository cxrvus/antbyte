use anyhow::{Result, anyhow};

use crate::ant::{
	compiler::{FlatAssignment, Normalizer, Wire},
	world::parser::{Assignment, Expression, ParsedCircuit},
};

impl Assignment {
	pub(super) fn flatten(&self, start_index: &mut u32) -> Vec<FlatAssignment> {
		let mut flat_assignments = flatten_expression(&self.expression, start_index);
		flat_assignments.last_mut().unwrap().assignees = self.assignees.clone();
		flat_assignments
	}
}

#[inline]
fn format_index(index: u32) -> String {
	format!("_exp{index:02}")
}

fn flatten_expression(exp: &Expression, index: &mut u32) -> Vec<FlatAssignment> {
	let mut flat_exps = vec![];

	let (call, wires) = if let Some(parameters) = &exp.parameters {
		let mut wires = vec![];

		for sub_exp in parameters {
			if sub_exp.parameters.is_some() {
				flat_exps.extend(flatten_expression(sub_exp, index));

				wires.push(Wire {
					sign: sub_exp.sign,
					target: format_index(*index - 1),
				});
			} else {
				wires.push(Wire {
					sign: sub_exp.sign,
					target: sub_exp.ident.clone(),
				});
			}
		}

		(exp.ident.clone(), wires)
	} else {
		let wires = vec![Wire {
			sign: exp.sign,
			target: exp.ident.clone(),
		}];

		("or".to_string(), wires)
	};

	flat_exps.push(FlatAssignment {
		call,
		assignees: vec![format_index(*index)],
		sign: exp.sign,
		wires,
	});

	*index += 1;
	flat_exps
}

impl Normalizer {
	pub(super) fn validate_assignments(
		&self,
		flat_assignments: &Vec<FlatAssignment>,
		circuit: &ParsedCircuit,
	) -> Result<()> {
		let ParsedCircuit {
			inputs, outputs, ..
		} = circuit;

		for flat_assignment in flat_assignments {
			for wire in flat_assignment.wires.iter() {
				let target = &wire.target;

				let is_in_input = inputs.contains(target);
				let is_declared = is_in_input
					|| flat_assignments
						.iter()
						.any(|x| x.assignees.contains(target));

				if !is_declared {
					let error = if self.0.contains_key(target) {
						anyhow!("'{target}' is a circuit, not an input")
					} else if outputs.contains(target) {
						anyhow!("'{target}' is an output, not an input")
					} else {
						anyhow!("unknown identifier: '{target}'")
					};

					return Err(error);
				}
			}
		}

		Ok(())
	}
}
