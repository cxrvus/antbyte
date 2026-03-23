use super::World;
use crate::plugins::render::{
	Renderer,
	term_render::{clear_screen, print_title_short},
};
use anyhow::Result;

const MAX_TICKS: u32 = 1 << 16;

impl World {
	pub fn run(&mut self, renderer: &mut impl Renderer) -> Result<()> {
		renderer.open();

		if self.config().looping {
			let properties = self.properties.clone();
			loop {
				let mut world = World::new(properties.clone())?;
				world.run_once(renderer);
			}
		} else {
			self.run_once(renderer);
			Ok(())
		}
	}

	fn run_once(&mut self, renderer: &mut impl Renderer) {
		if self.config().speed.is_some() {
			renderer.render(self);

			loop {
				let world_active = self.frame_tick();

				renderer.render(self);

				if !world_active {
					break;
				}
			}
		} else {
			self.instant_run()
		}

		renderer.render(self);
	}

	fn instant_run(&mut self) {
		let max_ticks = self.config().ticks.unwrap_or(MAX_TICKS);
		self.properties.config.ticks = Some(max_ticks);

		while self.tick() {
			if self.tick_count.is_multiple_of(0x100) {
				clear_screen();
				print_title_short();
				eprintln!("processing tick {} out of {max_ticks:0>4}", self.tick_str());
				eprintln!();
			}
		}
	}

	#[inline]
	pub fn tick_str(&self) -> String {
		format!("{:0>8}", self.tick_count())
	}
}
