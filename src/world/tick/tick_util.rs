use crate::{
	ant::Ant,
	util::{
		dir::Direction,
		vec2::{Pos, Vec2},
	},
	world::{World, config::BorderMode, state::Ants},
};

impl World {
	pub(super) fn next_pos(&self, pos: Pos, layer: u8, dir: Direction) -> Option<Pos> {
		let new_pos = pos.sign() + dir.as_vec();

		if self.tiles.in_bounds(&new_pos) {
			Some(new_pos.unsign().unwrap())
		} else {
			use BorderMode::*;

			let border_mode = self.border_mode(layer);

			match border_mode {
				Collide | Despawn => None,
				Cycle | Wrap => {
					let dimensions = self.tiles.dimensions().sign();

					let mut wrapped_pos = new_pos % dimensions;

					if let Wrap = border_mode {
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

	pub(super) fn set_tile(&mut self, pos: Pos, value: u8, mask: u8) {
		let old_value = self.tiles.get(pos).unwrap();
		let new_value = value | (old_value & !mask);
		self.set_value(pos, new_value);
	}

	fn set_value(&mut self, pos: Pos, value: u8) {
		if let Some(decay) = self.config().decay {
			if value != 0 {
				let clock = self.tick_count as u16;
				let expiration = clock.wrapping_add(decay);
				self.tile_decays.insert(pos, expiration);
			} else {
				self.tile_decays.remove(&pos);
			}
		}

		self.tiles.set(pos, value);
	}

	/// get positions of neighboring ants about to move to target
	pub(super) fn get_contestants(&self, source: &Ants, target_pos: Pos, layer: u8) -> Vec<Pos> {
		let mut positions = vec![];

		for dir in 0..=Direction::MAX {
			let dir = Direction::from(dir);

			if let Some(source_pos) = self.next_pos(target_pos, layer, dir.inverted())
				&& let Some(source_ant) = source.get(&source_pos)
				&& !source_ant.halted()
				&& source_ant.dir == dir
			{
				positions.push(source_pos);
			}
		}

		positions
	}

	pub(super) fn luck_check(&self, layer: u8, contestants: &[Ant], challenger: &Ant) -> bool {
		if contestants.len() == 1 {
			true
		} else {
			challenger.luck(self.tick_count, layer)
				== contestants
					.iter()
					.map(|ant| ant.luck(self.tick_count, layer))
					.max()
					.expect("luck check with no contestants")
		}
	}
}
