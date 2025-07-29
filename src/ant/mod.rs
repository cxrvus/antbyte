pub mod archetype;
pub mod peripherals;

use crate::{
	ant::archetype::*,
	util::vec2::{Vec2, Vec2u},
	world::{BorderMode, World},
};

// idea: improve scope (no pub fields)
#[derive(Clone, Copy, Default)]
pub struct Ant {
	pub archetype: u32,
	pub alive: bool,
	pub pos: Vec2u,
	/// cardinal direction - number between 0 and 3
	pub dir: u8,
	pub age: u32,
	pub memory: Register,
}

// todo: move methods to world
impl Ant {
	pub fn new(archetype: u32) -> Self {
		Self {
			archetype,
			alive: true,
			..Default::default()
		}
	}

	pub fn die(&mut self) {
		self.alive = false;
	}

	pub fn get_dir_vec(&self) -> Vec2 {
		assert!(self.dir < 4);
		Vec2::cardinal()[self.dir as usize]
	}

	pub fn set_dir(&mut self, dir: u8) {
		self.dir = dir % 4;
	}

	pub fn next_pos(&self, world: &World) -> Option<Vec2u> {
		let (pos, dir) = (self.pos.sign(), self.get_dir_vec());
		let new_pos = pos + dir;

		if world.cells.in_bounds(&new_pos) {
			Some(new_pos.unsign().unwrap())
		} else {
			use BorderMode::*;

			match world.border_mode() {
				Collide | Despawn => None,
			}
		}
	}

	pub fn move_tick(&mut self, world: &World) {
		if let Some(new_pos) = self.next_pos(world) {
			// ant collision check
			if !world.ants.iter().any(|ant| ant.pos == new_pos) {
				self.pos = new_pos;
			}
		} else if let BorderMode::Despawn = world.border_mode() {
			self.die();
		}
	}

	pub fn get_target_ant<'a>(&self, world: &'a mut World) -> Option<&'a mut Ant> {
		let pos = self.next_pos(world)?;
		world.ants.iter_mut().find(|ant| ant.pos == pos)
	}

	pub fn spawn(world: &mut World, archetype: u32, pos: Vec2u) {
		if world.get_archetype(archetype).is_some() {
			let mut ant = Ant::new(archetype);
			ant.pos = pos;
			world.ants.push(ant);
		}
	}
}
