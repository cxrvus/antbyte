use std::collections::BTreeMap;

use crate::{
	ant::{
		Ant,
		pin::{Pin, PinValue},
	},
	util::vec2::Vec2u,
	world::config::BorderMode,
};

use super::{Behavior, World};

use Pin::*;

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

impl World {
	pub(super) fn get_output(&mut self, ant: &Ant) -> Vec<PinValue> {
		let Behavior {
			inputs,
			outputs,
			logic: truth_table,
			..
		} = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let mut input_bits = 0u8;

		for input_sub_pin in inputs.iter() {
			let input_value: u8 = match input_sub_pin.pin {
				Time => ant.age(self.tick_count) as u8,
				Pulse => zero_count_mask(ant.age(self.tick_count) as u8),
				Cell => self.cells.at(&ant.pos.sign()).unwrap().value,
				Next => self
					.next_pos(ant)
					.map(|pos| self.cells.at(&pos.sign()).unwrap().value)
					.unwrap_or(0u8),
				Mem => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),
				Collide => match self.next_pos(ant) {
					Some(pos) => self.is_occupied(&pos).into(),
					None => 1,
				},
				Event => self.event_in,
				ExtIn => self.ext_input,
				_ => panic!("unhandled input: {input_sub_pin:?}"),
			};

			let bit_index = input_sub_pin.line;
			let masked_input_value = (input_value >> bit_index) & 1;
			input_bits <<= 1;
			input_bits |= masked_input_value;
		}

		// calculating the output
		let mut output_bits = truth_table.get(input_bits);

		// condense output bits into bytes
		let mut output_values: Vec<PinValue> = vec![];

		for output_sub_pin in outputs.iter().rev() {
			let output_bit = (output_bits & 1) as u8;
			let bit_index = output_sub_pin.line;
			let new_value = output_bit << bit_index;

			if let Some(output_value) = output_values
				.iter_mut()
				.find(|output_value| output_value.pin == output_sub_pin.pin)
			{
				output_value.value |= new_value;
			} else {
				output_values.push(PinValue {
					pin: output_sub_pin.pin,
					value: new_value,
				});
			}

			output_bits >>= 1;
		}

		output_values
	}

	pub(super) fn sync_tick(&mut self, ant_index: usize, outputs: &Vec<PinValue>) {
		let mut ant = self.ants[ant_index];

		let cell_mask = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID")
			.cell_mask();

		let mut clear = false;
		let mut halted = false;

		for PinValue { pin, value } in outputs {
			match (pin, value) {
				(Clear, 1) => clear = true,
				(Cell, _) => self.set_cell(&ant, *value, cell_mask),

				(AntDir, value) => ant.child_dir = *value,
				(AntMem, value) => ant.child_memory = *value,
				(Mem, value) => ant.memory = *value,

				(Send, value) => self.event_out |= value,
				(ExtOut, value) => self.ext_output.push(*value),

				// these are deferred to async ticks...
				// kill_tick
				(Kill, 1) => {
					if let Some(pos) = self.next_pos(&ant)
						&& let Some(index) = self.get_ant_index(&pos)
					{
						self.async_actions.kills.push(index);
					}
				}

				// move_tick
				(Dir, _) => ant.set_dir(ant.dir + value),
				(Halt, _) => halted = *value != 0,

				// spawn_tick
				(AntSpawn, _) if *value != 0 => {
					ant.child_behavior = *value;
					self.async_actions.spawns.push(ant_index);
				}

				// die_tick
				(Die, 1) => self.async_actions.deaths.push(ant_index),
				_ => {}
			};
		}

		if clear {
			self.set_cell(&ant, 0, !cell_mask);
		}

		// move_tick
		if !halted {
			self.async_actions.moves.push(ant_index);
		}

		self.ants[ant_index] = ant;
	}

	pub(super) fn kill_tick(&mut self) {
		for index in self.async_actions.clone().kills {
			self.kill(index);
		}
	}

	pub(super) fn die_tick(&mut self) {
		for index in self.async_actions.clone().deaths {
			self.kill(index);
		}
	}

	pub(super) fn move_tick(&mut self) {
		let mut claims = BTreeMap::<Vec2u, Vec<usize>>::new();
		let mut despawns = vec![];

		for index in &self.async_actions.moves.clone() {
			let ant = self.ants[*index];

			if !ant.is_alive() {
				continue;
			}

			if let Some(target) = self.next_pos(&ant) {
				self.occupy(&ant.pos, false);
				claims.entry(target).or_default().push(*index);
			} else if let BorderMode::Despawn = self.config().border_mode {
				despawns.push(*index);
			}
		}

		for index in despawns {
			self.kill(index);
		}

		for (target, indexes) in claims {
			// idea: customize conflict resolution strategy in config

			// conflict resolution
			let index = indexes.iter().min().unwrap();
			let mut ant = self.ants[*index];

			// conflict losers return to their original positions
			for other_index in &indexes {
				if other_index != index {
					let other_pos = &self.ants[*other_index].pos.clone();
					self.occupy(other_pos, true);
				}
			}

			// conflict winner moves to target position
			if !self.is_occupied(&target) {
				self.occupy(&target, true);
				ant.pos = target;
				self.ants[*index] = ant;
			} else {
				// conflict winner also returns to their original positions
				self.occupy(&ant.pos, true);
			}
		}
	}

	const ANT_LIMIT: u32 = 0x100;

	pub(super) fn spawn_tick(&mut self) {
		let mut claims = BTreeMap::<Vec2u, Vec<usize>>::new();

		for index in &self.async_actions.spawns {
			let ant = self.ants[*index];

			if !ant.is_alive() {
				continue;
			} else if let Some(target) = self.flipped_next_pos(&ant)
				&& self.get_behavior(ant.child_behavior).is_some()
			{
				// direction gets flipped, so that the new ant
				// spawns behind the old one and not in front of it
				claims.entry(target).or_default().push(*index);
			}
		}

		let ant_limit = self.config().ant_limit.unwrap_or(Self::ANT_LIMIT) as usize;

		for (target, indexes) in claims {
			if self.ants.len() >= ant_limit {
				break;
			} else if self.is_occupied(&target) {
				continue;
			}

			// idea: customize conflict resolution strategy in config
			// conflict resolution
			let index = indexes.iter().min().unwrap();
			let ant = self.ants[*index];

			let child_dir = Ant::wrap_dir(ant.dir + ant.child_dir);

			let new_ant = Ant {
				pos: target,
				behavior: ant.child_behavior,
				memory: ant.child_memory,
				dir: child_dir,
				birth_tick: self.tick_count,
				..Default::default()
			};

			self.spawn(new_ant);
		}
	}
}
