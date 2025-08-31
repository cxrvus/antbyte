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
		dbg!((&inputs, &outputs));
		println!("{func}");
		let logic = func.simulate();

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

		debug_assert_eq!(self.signature.params.len(), inputs.len());
		debug_assert_eq!(self.signature.assignees.len(), outputs.len());

		Ok((inputs, outputs))
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
				IoType::Output => variables.push(target.clone()),
				IoType::Input => return Err(anyhow!("unknown variable: {target}")),
			}
		}

		Ok(())
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

	fn simulate(&self) -> TruthTable {
		todo!()
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
			dbg!(&bits);
			let value = CompFunc::int_from_bits(&bits) as u8;
			dbg!(value);
			assert_eq!(i, value);
		}
	}
}
