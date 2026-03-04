use std::fmt::Display;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Default)]
pub struct TruthTable {
	input_count: u8,
	output_count: u8,
	entries: Vec<u32>,
}

impl TruthTable {
	pub fn new(input_bits: usize, output_bits: usize, entries: Vec<u32>) -> Result<Self> {
		if output_bits > 32 {
			Err(anyhow!("output bit count must not be greater than 32"))
		} else if entries.len() != 1 << input_bits {
			Err(anyhow!("entry count must be equal to [1 << input_bits]"))
		} else if let Some(index) = entries.iter().position(|x| *x > (1 << output_bits)) {
			Err(anyhow!(
				"all entries must not be greater than {} (1 << output_bits)\nfound {} at index {}",
				1 << output_bits,
				entries[index],
				index
			))
		} else {
			Ok(Self {
				input_count: input_bits as u8,
				output_count: output_bits as u8,
				entries,
			})
		}
	}

	pub fn input_count(&self) -> u8 {
		self.input_count
	}

	pub fn output_count(&self) -> u8 {
		self.output_count
	}

	pub fn entries(&self) -> &Vec<u32> {
		&self.entries
	}

	// idea: optimize - memory efficiency using bit shifting
	pub fn get(&self, input: u8) -> u32 {
		self.entries
			.get(input as usize)
			.copied()
			.unwrap_or_default()
	}
}

impl Display for TruthTable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (input, output) in self.entries.iter().enumerate() {
			writeln!(f, "{input:08b} => {output:08b}")?;
		}

		writeln!(f)
	}
}
