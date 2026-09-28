use std::{mem::swap, sync::LazyLock};

use crate::{
	ant::Ant,
	util::{
		dir::Direction,
		vec2::{Pos, PosOffset},
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
				} else if let Some(cycle_pos_value) = cycle_pos {
					if pos == cycle_pos_value {
						// reached last ant in cycle
						cycle_pos = None;
					}

					let target_pos = self
						.next_pos(pos, layer, ant.move_dir())
						.expect("no target position for ant in cycle");

					// all ants in cycle can move
					MoveAction::Move(target_pos)
				} else if let Some(target_pos) = self.next_pos(pos, layer, ant.move_dir()) {
					if result.contains_key(&target_pos) {
						// target pos is occupied in result => can't move
						MoveAction::Stay
					} else if let Some(&target_ant) = source.get(&target_pos) {
						// target pos is occupied in source
						if target_ant.halted() {
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
						BorderMode::Collide => MoveAction::Stay,
						BorderMode::Despawn => MoveAction::Nop,
						_ => panic!("no target position, despite border mode guaranteeing one"),
					}
				};

				match action {
					MoveAction::Move(target_pos) => {
						let mut ant = ant;
						ant.update_offset();
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

const OFFSET_SCALE: i8 = 64;

static OFFSET_TABLE: LazyLock<[PosOffset; 0x100]> = LazyLock::new(|| {
	std::array::from_fn(|dir| {
		let angle = (dir as f64) * (2.0 * std::f64::consts::PI / 256.0);

		let raw_dx = angle.cos();
		let raw_dy = angle.sin();

		// normalize so the larger axis == 1.0, then scale to SCALE
		let largest = raw_dx.abs().max(raw_dy.abs());

		let dx = ((raw_dx / largest) * OFFSET_SCALE as f64).round() as i8;
		let dy = ((raw_dy / largest) * OFFSET_SCALE as f64).round() as i8;

		PosOffset { x: dx, y: dy }
	})
});

impl Ant {
	pub fn new_offset(&self) -> PosOffset {
		let rot = self.rotation.value();
		let delta = OFFSET_TABLE[rot as usize];
		let x = add_delta(self.pos_offset.x, delta.x);
		let y = add_delta(self.pos_offset.y, delta.y);
		PosOffset { x, y }
	}

	pub fn update_move_dir(&mut self) {
		let offset = self.new_offset();
		let dir_x = dir_coord(offset.x);
		let dir_y = dir_coord(offset.y);
		let dir = coords_to_dir((dir_x, dir_y));
		self.move_dir = dir;
	}

	pub fn update_offset(&mut self) {
		let offset = self.new_offset();
		let x = clamp_offset(offset.x);
		let y = clamp_offset(offset.y);
		self.pos_offset = PosOffset { x, y };
	}
}

fn add_delta(offset: i8, delta: i8) -> i8 {
	if delta > 0 && offset < 0 || delta < 0 && offset > 0 {
		delta
	} else {
		offset + delta
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

fn coords_to_dir(vec: (i8, i8)) -> Direction {
	Direction::from(match vec {
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
	})
}
