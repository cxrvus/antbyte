use std::mem::swap;

use crate::{
	ant::Ant,
	util::{
		dir::Direction,
		vec2::{OFFSET_SCALE, Pos, PosOffset, PosOffsetTable},
	},
	world::{World, config::BorderMode, state::Ants},
};

enum MoveAction {
	Stay,
	Move(Pos),
	Nop,
}

impl World {
	pub(super) fn move_tick(&mut self, layer: u8) {
		let mut source = Ants::new();
		let mut result = Ants::new();

		swap(self.ants.layer_mut(layer), &mut source);

		while let Some((pos, ant)) = source.pop_first() {
			let mut stack = vec![(pos, ant)];

			// used to resolve cycles
			let mut cycle_pos: Option<Pos> = None;

			while let Some((pos, ant)) = stack.pop() {
				let action = if ant.halted() {
					MoveAction::Stay
				} else if ant.move_dir().is_none() {
					MoveAction::Move(pos)
				} else if let Some(dir) = ant.move_dir()
					&& let Some(cycle_pos_value) = cycle_pos
				{
					if pos == cycle_pos_value {
						// reached last ant in cycle
						cycle_pos = None;
					}

					let target_pos = self
						.next_pos(pos, layer, dir)
						.expect("no target position for ant in cycle");

					// all ants in cycle can move
					MoveAction::Move(target_pos)
				} else if let Some(dir) = ant.move_dir()
					&& let Some(target_pos) = self.next_pos(pos, layer, dir)
				{
					if result.contains_key(&target_pos) {
						// target pos is occupied in result => can't move
						MoveAction::Stay
					} else if let Some(&target_ant) = source.get(&target_pos) {
						// target pos is occupied in source
						if target_ant.halted() || target_ant.move_dir().is_none() {
							// dead end => stay
							MoveAction::Stay
						} else {
							// chain => recurse
							stack.push((pos, ant));
							source.remove(&target_pos);
							stack.push((target_pos, target_ant));
							MoveAction::Nop
						}
					} else {
						// target pos is free in source

						if stack.iter().any(|(visited, _)| target_pos == *visited) {
							// target is already part of the chain
							// cycle => resolve
							cycle_pos = Some(target_pos);
							MoveAction::Move(target_pos)
						} else {
							let contestants = self
								.get_contestants(&source, target_pos, layer)
								.iter()
								.map(|pos| source[pos])
								.collect::<Vec<_>>();

							if contestants.is_empty() || self.luck_check(layer, &contestants, &ant)
							{
								// target is uncontested or conflict has been won => move
								MoveAction::Move(target_pos)
							} else {
								// conflict has been lost => stay
								MoveAction::Stay
							}
						}
					}
				} else {
					// target pos is outside of grid
					match self.border_mode(layer) {
						BorderMode::Obs => MoveAction::Stay,
						BorderMode::Die => MoveAction::Nop,
						_ => panic!("no target position, despite border mode guaranteeing one"),
					}
				};

				match action {
					MoveAction::Move(target_pos) => {
						let mut ant = ant;
						ant.update_offset(&self.cache.offset_table());
						commit(&mut result, target_pos, ant);
					}
					MoveAction::Stay => commit(&mut result, pos, ant),
					MoveAction::Nop => { /* ant will not be committed to result */ }
				}
			}

			// reached end of ant chain
		}

		fn commit(result: &mut Ants, pos: Pos, ant: Ant) {
			let prev = result.insert(pos, ant);
			assert!(prev.is_none(), "tried to occupy occupied space")
		}

		swap(&mut result, self.ants.layer_mut(layer));
	}
}

impl Ant {
	pub fn new_offset(&self, offset_table: &PosOffsetTable) -> PosOffset {
		let rot = self.rotation.value();
		let delta = offset_table[rot as usize];
		let x = self.pos_offset.x + delta.x;
		let y = self.pos_offset.y + delta.y;
		PosOffset { x, y }
	}

	pub fn update_move_dir(&mut self, offset_table: &PosOffsetTable) {
		let offset = self.new_offset(offset_table);
		let dir_x = dir_coord(offset.x);
		let dir_y = dir_coord(offset.y);
		let dir = coords_to_dir((dir_x, dir_y));
		self.move_dir = dir;
	}

	pub fn update_offset(&mut self, offset_table: &PosOffsetTable) {
		let offset = self.new_offset(offset_table);
		let x = clamp_offset(offset.x);
		let y = clamp_offset(offset.y);
		self.pos_offset = PosOffset { x, y };
	}
}

fn dir_coord(offset_coord: i8) -> i8 {
	if offset_coord >= OFFSET_SCALE {
		1
	} else if offset_coord <= -OFFSET_SCALE {
		-1
	} else {
		0
	}
}

fn clamp_offset(offset_coord: i8) -> i8 {
	if offset_coord >= OFFSET_SCALE {
		offset_coord - OFFSET_SCALE
	} else if offset_coord <= -OFFSET_SCALE {
		offset_coord + OFFSET_SCALE
	} else {
		offset_coord
	}
}

fn coords_to_dir(vec: (i8, i8)) -> Option<Direction> {
	if vec == (0, 0) {
		None
	} else {
		Some(Direction::from_u8(match vec {
			(1, 0) => 0,
			(1, 1) => 1,
			(0, 1) => 2,
			(-1, 1) => 3,
			(-1, 0) => 4,
			(-1, -1) => 5,
			(0, -1) => 6,
			(1, -1) => 7,
			_ => panic!(
				"rotation returned an invalid direction vector: ({}, {})",
				vec.0, vec.1
			),
		}))
	}
}
