use crate::{
	ant::circuit::{self, Circuit, Layer, Weight},
	util::matrix::Matrix,
};
use anyhow::{Result, anyhow};

pub struct Parser;

impl Parser {
	pub fn parse(code: String) -> Result<Circuit> {
		let lines = code.trim().lines();

		let height = lines.clone().count();
		let width = lines.clone().next().unwrap().len();

		let mut weights: Vec<Weight> = vec![];

		for line in lines {
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

		let weight_matrix = Matrix::with_values(width, height, weights);
		let layers = vec![Layer::new(weight_matrix)];
		let circuit = Circuit::new(layers);

		Ok(circuit)
	}
}
