use crate::{ui::term::render::TermRenderer, util::sleep, world::World};
use std::{io, time::Instant};

pub mod keyboard;
pub mod raw;
pub mod render;

pub fn run(world: World, hide_title: bool) {
	let mut world = world;
	let renderer = TermRenderer::new(hide_title);

	// TODO: get ext input

	let mut last_frame = Instant::now();

	while let Some(frame) = world.next_frame_auto() {
		// wait before every frame (except frame 0)
		if world.tick_count() > 0 {
			if let Some(frame_ms) = frame.ms {
				// wait for frame interval to elapse
				let elapsed = last_frame.elapsed().as_millis() as u32;
				if elapsed < frame_ms {
					// add a small buffer to prevent flickering
					let sleep_ms = (frame_ms - elapsed).max(8);
					sleep(sleep_ms);
				}
				last_frame = Instant::now();
			} else {
				// wait for key input to continue
				eprintln!("<i> Press <Enter> to step to next frame");
				let mut input = String::new();
				io::stdin().read_line(&mut input).unwrap();
			}
		}

		renderer.render_frame(&world);
	}
}
