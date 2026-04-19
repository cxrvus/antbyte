use crate::{
	ant::Ant,
	util::vec2::{Vec2, Vec2u},
	world::{Cell, World, config::BorderMode},
};

impl World {
	pub(super) fn next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let (pos, dir) = (ant.pos.sign(), ant.dir);

		let _different_layer = false; // idea: spawning ants on different z-layers
		let new_pos = if _different_layer {
			pos
		} else {
			pos + dir.as_vec()
		};

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

	pub(super) fn flipped_next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let mut ant = *ant;
		ant.dir = ant.dir.inverted();
		self.next_pos(&ant)
	}

	pub(super) fn set_cell(&mut self, pos: &Vec2u, value: u8, mask: u8) {
		let old_value = self.cells.at(pos).unwrap().value;
		let new_value = value | (old_value & !mask);
		self.set_value(pos, new_value);
	}

	#[rustfmt::skip]
	fn set_value(&mut self, pos: &Vec2u, value: u8) {
		let old_cell = self.cells.at(pos).unwrap();

		let expiration = match self.config().decay {
			Some(decay) if value != 0 => {
				let clock = self.tick_count as u16;
				Some(clock.wrapping_add(decay))
			}
			_ => None
		};

		let cell = Cell { value, expiration, ..*old_cell };

		self.cells.set_at(pos, cell);
	}


	#[rustfmt::skip]
	#[inline]
	pub(super) fn occupy(&mut self, pos: &Vec2u, occupied: bool) {
		let mut cell = self.cells.at(pos).unwrap().clone();
		assert_ne!(cell.occupied, occupied);
		cell.occupied = occupied;
		self.cells.set_at(pos, cell);
	}

	#[inline]
	pub(super) fn is_occupied(&self, pos: &Vec2u) -> bool {
		self.cells
			.at(pos)
			.expect("position out of bounds: {pos:?}")
			.occupied
	}

	pub(super) fn get_ant_index(&self, pos: &Vec2u) -> Option<usize> {
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

	pub(super) fn spawn(&mut self, ant: Ant) {
		if !self.is_occupied(&ant.pos) {
			self.ants.push(ant);

			self.occupy(&ant.pos, true);
		}
	}

	pub(super) fn kill(&mut self, index: usize) {
		let ant = self.ants[index];
		if ant.is_alive() {
			self.ants[index].die();

			self.occupy(&ant.pos, false);
		}
	}
}
