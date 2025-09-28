use crate::{
	ant::{
		AntStatus, ColorMode,
		peripherals::{OutputValue, Peripheral},
	},
	util::vec2::Vec2u,
};

use super::{Ant, Behavior, BorderMode, World};

impl World {
	// idea: split up into sub-methods and rename
	pub(super) fn ant_tick(&mut self, ant_index: usize) {
		let ant = self.ants[ant_index];

		let Behavior {
			inputs,
			outputs,
			logic: truth_table,
			..
		} = self
			.get_behavior(ant.behavior)
			.clone()
			.expect("invalid Behavior ID");

		let mut input_bits = 0u8;

		use Peripheral::*;

		for input_spec in inputs.iter() {
			let input_value: u8 = match input_spec.peripheral {
				Time => ant.age as u8,
				Cell => *self.cells.at(&ant.pos.sign()).unwrap(),
				CellNext => self
					.next_pos(&ant)
					.map(|pos| *self.cells.at(&pos.sign()).unwrap())
					.unwrap_or(0u8),
				Memory => ant.memory,
				Random => self.rng(),
				Obstacle => match self.next_pos(&ant) {
					Some(pos) => self.is_occupied(&pos).into(),
					None => 1,
				},
				Direction => ant.dir,
				Halted => ant.halted as u8,
				_ => panic!("unhandled input: {input_spec:?}"),
			};

			let bit_index = input_spec.bit;
			let masked_input_value = (input_value >> bit_index) & 1;
			input_bits <<= 1;
			input_bits |= masked_input_value;
		}

		// calculating the output
		let mut output_bits = truth_table.get(input_bits);

		// condense output bits into bytes
		let mut output_values: Vec<OutputValue> = vec![];

		for output_spec in outputs.iter().rev() {
			let output_bit = (output_bits & 1) as u8;
			let bit_index = output_spec.bit;
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

			output_bits >>= 1;
		}

		output_values.sort();

		let mut ant = ant;

		for OutputValue { output, value } in output_values.into_iter() {
			match (output, value) {
				(Direction, _) => ant.set_dir(ant.dir + value),
				(Halted, _) => ant.halted = value != 0,
				(Cell, _) if value != 0 => {
					let adjusted = self.adjusted_color(value);
					self.cells.set_at(&ant.pos.sign(), adjusted);
				}
				(CellClear, 1) => self.cells.set_at(&ant.pos.sign(), 0),
				(Memory, value) => ant.memory = value,
				(SpawnAnt, _) if value != 0 => self.reproduce(&ant, value),
				(Kill, 1) => {
					if let Some(pos) = self.next_pos(&ant) {
						self.kill_at(&pos);
					}
				}
				(Die, 1) => self.die(&mut ant),
				_ => {}
			};
		}

		if ant.is_alive() && !ant.halted {
			self.move_tick(&mut ant);
		}

		ant.age += 1;

		self.ants[ant_index] = ant;
	}

	fn next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let (pos, dir) = (ant.pos.sign(), ant.dir_vec());
		let new_pos = pos + dir;

		if self.cells.in_bounds(&new_pos) {
			Some(new_pos.unsign().unwrap())
		} else {
			use BorderMode::*;

			match self.config().border_mode {
				Collide | Despawn => None,
				Cycle | Wrap => {
					let dimensions = self.cells.dimensions().sign();
					let mut wrapped_pos = new_pos % dimensions;

					if let Wrap = self.config().border_mode {
						if new_pos.x < 0 {
							wrapped_pos.x = dimensions.x - 1;
							wrapped_pos.y = (wrapped_pos.y - 1).rem_euclid(dimensions.y);
						} else if new_pos.x >= dimensions.x {
							wrapped_pos.x = 0;
							wrapped_pos.y = (wrapped_pos.y + 1).rem_euclid(dimensions.y);
						}

						if new_pos.y < 0 {
							wrapped_pos.y = dimensions.y - 1;
							wrapped_pos.x = (wrapped_pos.x - 1).rem_euclid(dimensions.x);
						} else if new_pos.y >= dimensions.y {
							wrapped_pos.y = 0;
							wrapped_pos.x = (wrapped_pos.x + 1).rem_euclid(dimensions.x);
						}
					}

					Some(wrapped_pos.unsign().unwrap())
				}
			}
		}
	}

	#[inline]
	pub fn is_occupied(&self, pos: &Vec2u) -> bool {
		*self
			.ant_cache
			.at(&pos.sign())
			.expect("position out of bounds: {pos:?}")
	}

	fn occupy(&mut self, pos: &Vec2u, value: bool) {
		self.ant_cache.set_at(&pos.sign(), value)
	}

	fn move_tick(&mut self, ant: &mut Ant) {
		if let Some(new_pos) = self.next_pos(ant) {
			if !self.is_occupied(&new_pos) {
				self.occupy(&new_pos, true);
				self.occupy(&ant.pos, false);
				ant.pos = new_pos;
			}
		} else if let BorderMode::Despawn = self.config().border_mode {
			self.die(ant);
		}
	}

	fn reproduce(&mut self, origin: &Ant, behavior_id: u8) {
		// direction gets flipped, so that the new ant
		// spawns behind the old one and not in front of her
		let original_dir = origin.dir;
		let mut ant = *origin;
		ant.flip_dir();

		if let Some(pos) = self.next_pos(&ant)
			&& self.get_behavior(behavior_id).is_some()
		{
			let new_ant = Ant::new(pos, original_dir, behavior_id);
			self.spawn(new_ant);
		}
	}

	const ANT_LIMIT: usize = 0x100;

	pub fn spawn(&mut self, ant: Ant) {
		if self.ants.len() < Self::ANT_LIMIT && !self.is_occupied(&ant.pos) {
			self.ants.push(ant);
			self.occupy(&ant.pos, true);
		}
	}

	fn get_ant_index(&self, pos: &Vec2u) -> Option<usize> {
		if self.is_occupied(pos) {
			Some(
				self.ants
					.iter()
					.position(|ant| ant.pos == *pos)
					.expect("couldn't find cached ant at {pos:?}"),
			)
		} else {
			None
		}
	}

	fn kill_at(&mut self, pos: &Vec2u) {
		if let Some(index) = self.get_ant_index(pos) {
			let ant_pos = self.ants[index].pos;
			self.ants[index].status = AntStatus::Dead;
			self.occupy(&ant_pos, false);
		}
	}

	fn die(&mut self, target: &mut Ant) {
		target.status = AntStatus::Dead;
		self.occupy(&target.pos, false);
	}

	fn adjusted_color(&self, color: u8) -> u8 {
		match self.config().color_mode {
			ColorMode::Binary => match color {
				0 => 0x0,
				_ => 0xf,
			},
			ColorMode::RGBI => color,
		}
	}
}
