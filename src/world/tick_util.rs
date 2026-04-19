use crate::{
	ant::Ant,
	util::vec2::Vec2u,
	world::{Cell, World, config::BorderMode},
};

impl World {
	pub(super) fn next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let (pos, dir) = (ant.pos.sign(), ant.dir_vec());

		let _different_layer = false; // idea: spawning ants on different z-layers
		let new_pos = if _different_layer { pos } else { pos + dir };

		if self.cells.in_bounds(&new_pos) {
			Some(new_pos.unsign().unwrap())
		} else {
			use BorderMode::*;

			match self.config().border_mode {
				Collide | Despawn => None,
				Cycle | Wrap => {
					let dimensions = self.cells.dimensions().sign();
					let mut wrapped_pos = new_pos % dimensions;
					let is_diagonal = dir.x != 0 && dir.y != 0;

					if let Wrap = self.config().border_mode
						&& !is_diagonal
					{
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

	pub(super) fn flipped_next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let mut ant = *ant;
		ant.flip_dir();
		self.next_pos(&ant)
	}

	pub(super) fn set_cell(&mut self, ant: &Ant, value: u8, mask: u8) {
		let old_value = self.cells.at(&ant.pos.sign()).unwrap().value;
		let new_value = value | (old_value & !mask);
		self.set_value(&ant.pos, new_value);
	}

	#[rustfmt::skip]
	fn set_value(&mut self, pos: &Vec2u, value: u8) {
		let old_cell = self.cells.at(&pos.sign()).unwrap();

		let expiration = match self.config().decay {
			Some(decay) if value != 0 => {
				let clock = self.tick_count as u16;
				Some(clock.wrapping_add(decay))
			}
			_ => None
		};

		let cell = Cell { value, expiration, ..*old_cell };

		self.cells.set_at(&pos.sign(), cell);
	}


	#[rustfmt::skip]
	#[inline]
	pub(super) fn occupy(&mut self, pos: &Vec2u, occupied: bool) {
		let mut cell = self.cells.at(&pos.sign()).unwrap().clone();
		assert_ne!(cell.occupied, occupied);
		cell.occupied = occupied;
		self.cells.set_at(&pos.sign(), cell);
	}

	#[inline]
	pub(super) fn is_occupied(&self, pos: &Vec2u) -> bool {
		self.cells
			.at(&pos.sign())
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
