use crate::world::World;

impl World {
	pub fn frame_tick(&mut self) -> bool {
		for _ in 0..self
			.config()
			.speed
			.expect("speed must be greater than 0 to use frame_tick")
		{
			if !self.tick() {
				return false;
			}
		}

		true
	}

	pub fn tick(&mut self) -> bool {
		self.tick_count += 1;

		self.event_in = self.event_out;
		self.event_out = 0;

		if self.queen.is_some() {
			self.ant_tick(None);
		}

		for i in 0..self.ants.len() {
			if self.ants[i].is_alive() {
				self.ant_tick(Some(i));
			}
		}

		// todo: optimize defragmentation
		self.ants.iter_mut().for_each(|ant| ant.grow_up());
		self.ants.retain(|ant| ant.is_alive());

		if let Some(queen) = self.queen
			&& !queen.is_alive()
		{
			self.queen = None;
		}

		// idea: optimize decay
		if self.config().decay.is_some() {
			self.cell_decay();
		}

		let no_ants = self.ants.is_empty() && self.queen.is_none();

		let tick_overflow = self
			.config()
			.ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		!(no_ants || tick_overflow)
	}
}
