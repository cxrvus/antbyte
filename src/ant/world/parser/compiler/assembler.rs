use std::collections::HashMap;

use crate::{
	ant::{
		Behavior,
		compiler::CompFunc,
		peripherals::{IoType, PeripheralBit},
		world::parser::{ParamValue, Signature, token::Token},
	},
	truth_table::TruthTable,
};

use anyhow::{Result, anyhow};

impl CompFunc {
	pub fn assemble(&self) -> Result<Behavior> {
		let mut func = self.clone();

		let (inputs, outputs) = func.extract_peripherals()?;
		// dbg!((&inputs, &outputs));
		let logic = func.simulate();
		// println!("{logic}");

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
			Self::extract_peripheral(
				&mut self.signature,
				&mut variables,
				&mut outputs,
				&mut statement.assignee,
				IoType::Output,
			)?;

			for param in &mut statement.params {
				Self::extract_peripheral(
					&mut self.signature,
					&mut variables,
					&mut inputs,
					param,
					IoType::Input,
				)?;
			}
		}

		inputs.reverse();

		if inputs.len() > 8 {
			Err(anyhow!(
				"may not have more than 8 inputs, got {}",
				inputs.len()
			))
		} else if outputs.len() > 32 {
			Err(anyhow!(
				"may not have more than 32 inputs, got {}",
				outputs.len()
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
			let periph = PeripheralBit::from_ident(target)?;

			if let Some(req_io) = periph.peripheral.properties().io_type
				&& req_io != io_type
			{
				return Err(anyhow!("cannot use peripheral '{target}' as {io_type:?}"));
			}

			*target = format_periph(target, io_type);

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
					return Err(anyhow!("unknown variable: {target}"));
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

	fn bits_from_int(value: u8) -> Vec<bool> {
		let mut bits = vec![];

		for i in 0..8 {
			bits.push((value >> i & 1) == 1);
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
