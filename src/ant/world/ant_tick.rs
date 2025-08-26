use crate::{
	ant::peripherals::{OutputValue, Peripheral},
	util::vec2::Vec2u,
};

use super::{Ant, Behavior, BorderMode, WorldInstance};

impl WorldInstance {
	// TODO: split up into sub-methods
	pub(super) fn ant_tick(&mut self, ant: &Ant) -> Ant {
		let world_image = self.clone();

		let Behavior {
			inputs,
			outputs,
			truth_table,
			..
		} = world_image
			.get_behavior(ant.behavior)
			.expect("invalid Behavior ID");

		let mut input_bits = 0u8;

		use Peripheral::*;

		for input_spec in inputs.iter() {
			let input_value: u8 = match input_spec.peripheral {
				Time => ant.age as u8,
				Cell => *self.cells.at(&ant.pos.sign()).unwrap(),
				CellNext => self
					.next_pos(ant)
					.map(|pos| *self.cells.at(&pos.sign()).unwrap())
					.unwrap_or(0u8),
				Memory => ant.memory,
				Random => self.rng(),
				Obstacle => self.get_target_ant(ant).is_some().into(), // todo: also true if at border
				Direction => ant.dir,
				Moving => ant.moving as u8,
				_ => panic!("unhandled input"),
			};

			let bit_index = input_spec.bit;
			let masked_input_value = (input_value >> bit_index) & 1;
			input_bits <<= 1;
			input_bits |= masked_input_value;
		}

		// calculating the output
		let mut output_bits = truth_table.get(input_bits);

		let mut ant = *ant;

		// condense output bits into bytes
		let mut output_values: Vec<OutputValue> = vec![];

		for output_spec in outputs.iter() {
			let bit_index = output_spec.bit;
			let output_bit = (output_bits & 1) as u8;
			let new_value = output_bit << bit_index;

			if let Some(output_value) = output_values
				.iter_mut()
				.find(|output_value| output_value.output == output_spec.peripheral)
			{
				output_value.value |= new_value;
			} else {
				output_values.push(OutputValue {
					output: output_spec.peripheral.clone(),
					value: new_value,
				});
			}

			output_bits >>= bit_index;
		}

		output_values.sort();

		for OutputValue { output, value } in output_values.into_iter() {
			match output {
				Direction => ant.set_dir(ant.dir + value),
				Moving => {
					let moving = value != 0;
					ant.moving = moving;

					if moving {
						self.move_tick(&mut ant);
					}
				}
				Cell if value != 0 => self.cells.set_at(&ant.pos.sign(), value),
				CellClear if value == 1 => self.cells.set_at(&ant.pos.sign(), 0),
				Memory if value != 0 => ant.memory = value,
				MemoryClear if value == 1 => ant.memory = 0,
				SpawnAnt => {
					// direction gets flip, so that new ant
					// spawns behind the queen and not in front of her
					ant.flip_dir();

					if let Some(pos) = self.next_pos(&ant)
						&& value > 0
					{
						Self::spawn(self, value - 1, pos);
					}

					ant.flip_dir();
				}
				Kill => {
					if let Some(ant) = self.get_target_ant(&ant) {
						ant.die();
					}
				}
				Die => ant.die(),
				_ => panic!("unhandled output"),
			};
		}

		ant
	}

	fn next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let (pos, dir) = (ant.pos.sign(), ant.get_dir_vec());
		let new_pos = pos + dir;

		if self.cells.in_bounds(&new_pos) {
			Some(new_pos.unsign().unwrap())
		} else {
			use BorderMode::*;

			match self.config.border_mode {
				Collide | Despawn => None,
			}
		}
	}

	fn move_tick(&self, ant: &mut Ant) {
		if let Some(new_pos) = self.next_pos(ant) {
			// ant collision check
			if !self.ants.iter().any(|ant| ant.pos == new_pos) {
				ant.pos = new_pos;
			}
		} else if let BorderMode::Despawn = self.config.border_mode {
			ant.die();
		}
	}

	fn get_target_ant<'a>(&'a mut self, ant: &Ant) -> Option<&'a mut Ant> {
		let pos = self.next_pos(ant)?;
		self.ants.iter_mut().find(|ant| ant.pos == pos)
	}

	fn spawn(&mut self, behavior: u8, pos: Vec2u) {
		if self.get_behavior(behavior).is_some() {
			let mut ant = Ant::new(behavior);
			ant.pos = pos;
			self.ants.push(ant);
		}
	}
}
