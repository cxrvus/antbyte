use std::collections::{BTreeMap, BTreeSet};

use crate::{
	ant::{
		Ant,
		pin::{Pin, PinValue},
	},
	util::{dir::Direction, vec2::Vec2u},
	world::config::BorderMode,
};

use super::{Behavior, World};

use Pin::*;

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

impl World {
	pub(super) fn get_output(&mut self, ant: &Ant, pos: Vec2u) -> Vec<PinValue> {
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
				Cell => self.cells.at(pos).unwrap().value,
				Next => self
					.next_pos(pos, ant.dir)
					.map(|pos| self.cells.at(pos).unwrap().value)
					.unwrap_or(0u8),
				Mem => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),
				Collide => match self.next_pos(pos, ant.dir) {
					Some(pos) => self.ants.get(&pos).is_some().into(),
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

	pub(super) fn sync_tick(&mut self, pos: Vec2u, outputs: &Vec<PinValue>) {
		let mut ant = self.ants[&pos];

		let cell_mask = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID")
			.cell_mask();

		let mut clear = false;

		for PinValue { pin, value } in outputs {
			match (pin, value) {
				(Clear, 1) => clear = true,
				(Cell, _) => self.set_cell(pos, *value, cell_mask),

				(AntDir, value) => ant.child_dir = Direction::new(*value),
				(AntMem, value) => ant.child_memory = *value,
				(Mem, value) => ant.memory = *value,

				(Send, value) => self.event_out |= value,
				(ExtOut, value) => self.ext_output.push(*value),

				// deferred to async ticks...

				// kill_tick
				(Kill, value) => ant.kill = *value != 0,

				// move_tick
				(Halt, _) => ant.halt = *value != 0,
				(Dir, _) => ant.dir += Direction::new(*value),

				// spawn_tick
				(AntSpawn, _) => ant.child_behavior = *value,

				// die_tick
				(Die, 1) => ant.die = true,
				_ => {}
			};
		}

		if clear {
			self.set_cell(pos, 0, !cell_mask);
		}

		self.ants.insert(pos, ant);
	}

	pub(super) fn kill_tick(&mut self) {
		let mut kills = BTreeSet::new();

		for (pos, ant) in &self.ants.clone() {
			if ant.kill
				&& let Some(next_pos) = self.next_pos(*pos, ant.dir)
				&& self.ants.contains_key(&next_pos)
			{
				kills.insert(next_pos);
			}
		}

		self.ants.retain(|pos, _| !kills.contains(pos));
	}

	pub(super) fn die_tick(&mut self) {
		self.ants.retain(|_, ant| !ant.die);
	}

	pub(super) fn move_tick(&mut self) {
		let mut source: BTreeMap<Vec2u, Ant> = self
			.ants
			.iter()
			.filter(|(_, ant)| !ant.halt)
			.map(|(pos, ant)| (*pos, *ant))
			.collect();

		let mut result: BTreeMap<Vec2u, Ant> = self
			.ants
			.iter()
			.filter(|(_, ant)| ant.halt)
			.map(|(pos, ant)| (*pos, *ant))
			.collect();

		let queue: Vec<Vec2u> = source.keys().cloned().collect();

		for pos in queue {
			let mut ant = match source.get(&pos) {
				Some(ant) => *ant,
				None => continue,
			};

			let mut stack = vec![pos];

			let mut cycle_pos: Option<Vec2u> = None;

			while let Some(pos) = stack.pop() {
				if let Some(cycle_pos_value) = cycle_pos {
					if pos == cycle_pos_value {
						cycle_pos = None;
					}
				} else if let Some(target_pos) = self.next_pos(pos, ant.dir) {
					if result.contains_key(&target_pos) {
						// target pos is occupied in result => can't move
						source.remove(&pos);
						result.insert(pos, ant);
					}

					if source.contains_key(&target_pos) {
						// target pos is occupied in source
						if stack.contains(&target_pos) {
							// cycle => resolve all ants up to target pos
							cycle_pos = Some(target_pos);
						} else {
							// chain => recurse
							stack.push(target_pos);
						}
					} else {
						// target pos is free in source
						// resolve conflict if free pos is contested
						let neighbors = self.get_contestants(source, target_pos);
						let (winner, losers) = self.resolve_conflict(neighbors);
						// TODO: CONTINUE: get positions instead of ants

						for loser in losers {}

						ant = winner;
					}
				} else {
					// target pos is outside of grid => die
					source.remove(&pos);
					continue;
				}

				// move
				let target_pos = self.next_pos(pos, ant.dir).unwrap();
				source.remove(&pos);
				result.insert(target_pos, ant);
			}
		}

		self.ants = result;
	}

	const ANT_LIMIT: u32 = 0x100;

	pub(super) fn spawn_tick(&mut self) {
		let mut claims = BTreeMap::<Vec2u, Vec<Vec2u>>::new();

		for (pos, ant) in &self.ants {
			if ant.child_behavior == 0 {
				continue;
			} else if let Some(target_pos) = self.next_pos(*pos, ant.dir.inverted())
				&& self.get_behavior(ant.child_behavior).is_some()
			{
				// direction gets flipped, so that the new ant
				// spawns behind the old one and not in front of it
				claims.entry(target_pos).or_default().push(target_pos);
			}
		}

		let ant_limit = self.config().ant_limit.unwrap_or(Self::ANT_LIMIT) as usize;

		for (target_pos, ants) in claims {
			if self.ants.len() >= ant_limit {
				break;
			} else if self.ants.contains_key(&target_pos) {
				continue;
			}

			// conflict resolution
			let (ant, _) = self.resolve_conflict(ants);

			let child_dir = ant.dir + ant.child_dir;

			let new_ant = Ant {
				behavior: ant.child_behavior,
				memory: ant.child_memory,
				dir: child_dir,
				birth_tick: self.tick_count,
				..Default::default()
			};

			self.ants.insert(target_pos, new_ant);
		}
	}
}
