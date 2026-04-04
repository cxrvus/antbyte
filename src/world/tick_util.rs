use crate::{
	ant::{Ant, AntStatus},
	util::vec2::Vec2u,
	world::{
		World,
		config::{BorderMode, ColorMode},
	},
};

impl World {
	pub(super) fn next_pos(&self, ant: &Ant) -> Option<Vec2u> {
		let (pos, dir) = (ant.pos.sign(), ant.dir_vec());
		let new_pos = if ant.is_queen() { pos } else { pos + dir };

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

	#[inline]
	pub(super) fn is_occupied(&self, pos: &Vec2u) -> bool {
		self.cells
			.at(&pos.sign())
			.expect("position out of bounds: {pos:?}")
			.occupied
	}

	pub(super) fn move_tick(&mut self, ant: &mut Ant) {
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

	pub(super) fn reproduce(
		&mut self,
		origin: &Ant,
		behavior_id: u8,
		child_dir: u8,
		child_mem: u8,
	) {
		let original_dir = origin.dir;
		let child_dir = Ant::wrap_dir(child_dir + original_dir);

		let mut ant = *origin;

		// direction gets flipped, so that the new ant
		// spawns behind the old one and not in front of it
		ant.flip_dir();

		if let Some(pos) = self.next_pos(&ant)
			&& self.get_behavior(behavior_id).is_some()
		{
			let new_ant = Ant {
				pos,
				behavior: behavior_id,
				memory: child_mem,
				dir: child_dir,
				..Default::default()
			};

			self.spawn(new_ant);
		}
	}

	pub(super) const ANT_LIMIT: u32 = 0x100;

	pub(super) fn spawn(&mut self, ant: Ant) {
		let ant_limit = self.config().ant_limit.unwrap_or(Self::ANT_LIMIT) as usize;
		if self.ants.len() < ant_limit && !self.is_occupied(&ant.pos) {
			self.ants.push(ant);
			self.occupy(&ant.pos, true);
		}
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

	pub(super) fn kill_at(&mut self, pos: &Vec2u) {
		if let Some(index) = self.get_ant_index(pos) {
			let ant_pos = self.ants[index].pos;
			self.ants[index].status = AntStatus::Dead;
			self.occupy(&ant_pos, false);
		}
	}

	pub(super) fn die(&mut self, target: &mut Ant) {
		target.status = AntStatus::Dead;
		self.occupy(&target.pos, false);
	}

	pub(super) fn adjusted_color(&self, color: u8) -> u8 {
		match self.config().color_mode {
			ColorMode::Binary => match color {
				0 => 0x0,
				_ => 0xf,
			},
			ColorMode::RGBI => color,
		}
	}
}
