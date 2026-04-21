use std::collections::BTreeMap;

use crate::{
	ant::Ant,
	util::{
		dir::Direction,
		vec2::{Vec2, Vec2u},
	},
	world::{Cell, World, config::BorderMode},
};

impl World {
	pub(super) fn next_pos(&self, pos: Vec2u, dir: Direction) -> Option<Vec2u> {
		let _different_layer = false; // idea: spawning ants on different z-layers
		let new_pos = if _different_layer {
			pos.sign()
		} else {
			pos.sign() + dir.as_vec()
		};

		if self.cells.in_bounds(&new_pos) {
			Some(new_pos.unsign().unwrap())
		} else {
			use BorderMode::*;

			match self.config().border_mode {
				Collide | Despawn => None,
				Cycle | Wrap => {
					let dimensions = Vec2u {
						x: self.config().width,
						y: self.config().height,
					}
					.sign();

					let mut wrapped_pos = new_pos % dimensions;

					if let Wrap = self.config().border_mode {
						let (size_x, size_y) = (dimensions.x, dimensions.y);
						let (new_x, new_y) = (new_pos.x, new_pos.y);
						let (mut wrapped_x, mut wrapped_y) = (wrapped_pos.x, wrapped_pos.y);

						if new_x < 0 {
							wrapped_x = size_x - 1;
							wrapped_y = (wrapped_y - 1).rem_euclid(size_y);
						} else if new_x >= size_x {
							wrapped_x = 0;
							wrapped_y = (wrapped_y + 1).rem_euclid(size_y);
						}

						if new_y < 0 {
							wrapped_y = size_y - 1;
							wrapped_x = (wrapped_x - 1).rem_euclid(size_x);
						} else if new_y >= size_y {
							wrapped_y = 0;
							wrapped_x = (wrapped_x + 1).rem_euclid(size_x);
						}

						wrapped_pos = Vec2 {
							x: wrapped_x,
							y: wrapped_y,
						}
					}

					Some(wrapped_pos.unsign().unwrap())
				}
			}
		}
	}

	pub(super) fn set_cell(&mut self, pos: Vec2u, value: u8, mask: u8) {
		let old_value = self.cells.at(pos).unwrap().value;
		let new_value = value | (old_value & !mask);
		self.set_value(pos, new_value);
	}

	#[rustfmt::skip]
	fn set_value(&mut self, pos: Vec2u, value: u8) {
		let expiration = match self.config().decay {
			Some(decay) if value != 0 => {
				let clock = self.tick_count as u16;
				Some(clock.wrapping_add(decay))
			}
			_ => None
		};

		let cell = Cell { value, expiration };

		self.cells.set_at(pos, cell);
	}

	/// gets positions of neighboring ants, that target position
	pub(super) fn get_contestants(
		&self,
		all_ants: &BTreeMap<Vec2u, Ant>,
		target_pos: Vec2u,
	) -> Vec<Vec2u> {
		let mut positions = vec![];

		for dir in 0..Direction::MAX {
			if let Some(source_pos) = self.next_pos(target_pos, Direction::new(dir).inverted())
				&& all_ants.contains_key(&source_pos)
			{
				positions.push(source_pos);
			}
		}

		positions
	}

	/// returns index of conflict winning ant
	pub(super) fn get_winner(&self, ants: &[Ant]) -> usize {
		if ants.len() == 1 {
			0
		} else {
			let mut max_luck = 0;
			let mut winner_index = 0;

			for (i, ant) in ants.iter().enumerate() {
				// CONTINUE
				let luck = ant.luck(self.tick_count);

				if luck > max_luck {
					max_luck = luck;
					winner_index = i;
				}
			}

			winner_index
		}
	}
}
