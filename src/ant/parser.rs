use crate::{
	ant::circuit::{Circuit, Layer, Weight},
	util::matrix::Matrix,
};
use anyhow::{Result, anyhow};

pub struct Parser;

impl Parser {
	pub fn parse(code: String) -> Result<Circuit> {
		// ';' can be used in place of a linebreak
		let code = code.replace(";", "\n");

		let sections = code.split("\n\n");
		let mut layers: Vec<Layer> = vec![];

		for section in sections {
			let layer = Self::parse_layer(section)?;
			layers.push(layer);
		}

		let circuit = Circuit::new(layers);
		Ok(circuit)
	}

	fn parse_layer(matrix_str: &str) -> Result<Layer> {
		let lines = matrix_str.trim().lines();
		let height = lines.clone().count();
		let width = lines.clone().next().unwrap().len();

		let mut weights: Vec<Weight> = vec![];

		for line in lines {
			let line = line.trim();

			for symbol in line.chars() {
				let weight = match symbol {
					'.' => Ok(Weight::Zero),
					'+' => Ok(Weight::Pos),
					'-' => Ok(Weight::Neg),
					other => Err(anyhow!("unknown weight symbol: {other}")),
				}?;

				weights.push(weight);
			}
		}

		let layer = Layer::new(Matrix::with_values(width, height, weights));
		Ok(layer)
	}
}
