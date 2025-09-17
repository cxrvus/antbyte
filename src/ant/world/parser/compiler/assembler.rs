use std::collections::HashMap;

use crate::{
	ant::{
		Behavior,
		compiler::{CompFunc, LogConfig},
		peripherals::{IoType, PeripheralBit},
		world::parser::{ParamValue, Signature, token::Token},
	},
	truth_table::TruthTable,
};

use anyhow::{Result, anyhow, bail};

impl CompFunc {
	pub fn assemble(&self, log_cfg: &LogConfig) -> Result<Behavior> {
		let mut func = self.clone();

		let (inputs, outputs) = func.extract_peripherals()?;

		// dbg!((&inputs, &outputs));

		if log_cfg.all {
			println!("\n{func}");
		}

		let logic = func.simulate();

		if log_cfg.all {
			println!("{logic}");
		}

		let behavior = Behavior {
			name: self.signature.name.clone(),
			logic,
			inputs,
			outputs,
		};

		Ok(behavior)
	}

	fn extract_peripherals(&mut self) -> Result<(Vec<PeripheralBit>, Vec<PeripheralBit>)> {
		let mut inputs: Vec<PeripheralBit> = vec![];
		let mut outputs: Vec<PeripheralBit> = vec![];
		let mut variables: Vec<String> = vec![];

		for statement in &mut self.comp_statements {
			for param in &mut statement.params {
				Self::extract_peripheral(
					&mut self.signature,
					&mut variables,
					&mut inputs,
					param,
					IoType::Input,
				)?;
			}

			Self::extract_peripheral(
				&mut self.signature,
				&mut variables,
				&mut outputs,
				&mut statement.assignee,
				IoType::Output,
			)?;
		}

		inputs.reverse();

		if inputs.len() > 8 {
			Err(anyhow!(
				"may not have more than 8 inputs, got {}\n{:?}:\n",
				inputs.len(),
				inputs
			))
		} else if outputs.len() > 32 {
			Err(anyhow!(
				"may not have more than 32 inputs, got {}\n{:?}:\n",
				outputs.len(),
				outputs
			))
		} else {
			Ok((inputs, outputs))
		}
	}

	fn extract_peripheral(
		signature: &mut Signature,
		variables: &mut Vec<String>,
		periphs: &mut Vec<PeripheralBit>,
		param: &mut ParamValue,
		io_type: IoType,
	) -> Result<()> {
		let target = &mut param.target;

		if Token::is_uppercase_ident(target) {
			let original_target = target.clone();

			if io_type == IoType::Input {
				let reassigned_output_name = format_periph(&original_target, IoType::Output);

				if signature.assignees.contains(&reassigned_output_name) {
					*target = reassigned_output_name;
					return Ok(());
				}
			}

			let periph = PeripheralBit::from_ident(&original_target)?;

			if let Some(req_io) = periph.peripheral.properties().io_type
				&& req_io != io_type
			{
				return Err(match req_io {
					IoType::Input => anyhow!("cannot assign to input-only peripheral '{target}'"),
					IoType::Output => anyhow!(
						"cannot use output-only peripheral '{target}' like an input\n(except if it has been assigned a value before)"
					),
				});
			}

			*target = format_periph(&original_target, io_type);

			if !periphs.contains(&periph) {
				periphs.push(periph);

				let signature_periphs = match io_type {
					IoType::Input => &mut signature.params,
					IoType::Output => &mut signature.assignees,
				};

				signature_periphs.push(target.clone());
			}
		} else if !variables.contains(target) {
			match io_type {
				IoType::Output if !signature.assignees.contains(target) => {
					variables.push(target.clone())
				}
				IoType::Input if !signature.params.contains(target) => {
					bail!("unknown variable: {target}");
				}
				_ => {}
			}
		}

		Ok(())
	}

	fn simulate(&self) -> TruthTable {
		let input_bits = self.signature.params.len();
		let output_bits = self.signature.assignees.len();
		let max_input = 1u8.unbounded_shl(input_bits as u32).wrapping_sub(1);

		let mut entries = vec![];

		for input in 0..=max_input {
			entries.push(self.tick(input));
		}

		TruthTable::new(input_bits, output_bits, entries).unwrap()
	}

	fn tick(&self, input: u8) -> u32 {
		let mut variables = HashMap::<String, bool>::new();
		let input_bits = Self::bits_from_int(input);

		for (n, input) in self.signature.params.iter().enumerate() {
			variables.insert(input.clone(), input_bits[n]);
		}

		for statement in &self.comp_statements {
			let mut assignee_value = statement.assignee.sign;

			for param in &statement.params {
				debug_assert!(
					variables.contains_key(&param.target),
					"unknown variable: {}",
					&param.target
				);

				let param_value = param.sign ^ variables[&param.target];

				if param_value {
					assignee_value = !statement.assignee.sign;
					break;
				}
			}

			variables.insert(statement.assignee.target.clone(), assignee_value);
		}

		let mut output_bits = vec![];

		for output in &self.signature.assignees {
			output_bits.push(variables[output]);
		}

		Self::int_from_bits(&output_bits)
	}

	fn bits_from_int(value: u8) -> [bool; 8] {
		let mut bits = [false; 8];

		for (i, bit) in bits.iter_mut().enumerate() {
			*bit = (value >> i & 1) == 1;
		}

		bits
	}

	fn int_from_bits(bits: &[bool]) -> u32 {
		let mut value = 0;

		for (i, &bit) in bits.iter().enumerate() {
			if bit {
				value |= 1 << i;
			}
		}

		value
	}
}

fn format_periph(ident: &str, io_type: IoType) -> String {
	let ident = ident.to_ascii_lowercase();

	let prefix = match io_type {
		IoType::Input => "i",
		IoType::Output => "o",
	};

	format!("_{prefix}_{ident}")
}

#[cfg(test)]
mod test {
	use crate::ant::compiler::CompFunc;

	#[test]
	fn bit_conversion() {
		for i in 0..=0xff {
			let bits = CompFunc::bits_from_int(i);
			let value = CompFunc::int_from_bits(&bits) as u8;
			assert_eq!(i, value);
		}
	}
}
