use crate::ant::circuit::{Circuit, Layer, WireArray};
use anyhow::{Result, anyhow};

pub struct Parser;

impl Parser {
	pub fn parse(code: String) -> Result<Circuit> {
		// ';' can be used in place of a linebreak
		let code = code.replace(';', "\n");

		let sections = code.split("\n\n");
		let mut layers: Vec<Layer> = vec![];

		for section in sections {
			let layer = Self::parse_layer(section)?;
			layers.push(layer);
		}

		let input_count = code.find('\n').unwrap_or(code.len());
		let circuit = Circuit::new(input_count, layers);
		Ok(circuit)
	}

	fn parse_layer(matrix_str: &str) -> Result<Layer> {
		let lines = matrix_str.trim().lines();
		let neuron_count = lines.clone().count() as u8;

		assert!(neuron_count > 0);
		assert!(neuron_count <= 32);

		let mut wire_matrix: Vec<WireArray> = vec![];

		for line in lines {
			let line = line.trim();

			assert!(line.chars().count() > 0);
			assert!(line.chars().count() <= 32);

			let mut inversion = 0u32;
			let mut mask = 0u32;

			for (i, symbol) in line.chars().enumerate() {
				let wire_bits = match symbol {
					'.' => Ok((0, 0)),
					'+' => Ok((0, 1)),
					'-' => Ok((1, 1)),
					other => Err(anyhow!("unknown wire symbol: {other}")),
				}?;

				inversion |= wire_bits.0 << i;
				mask |= wire_bits.1 << i;
			}

			let wires = WireArray::new(inversion, mask);
			wire_matrix.push(wires);
		}

		let layer = Layer::new(wire_matrix);
		Ok(layer)
	}
}
