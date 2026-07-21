#[cfg(feature = "midi")]
use anyhow::Context;

use anyhow::Result;

use crate::{
	ui::term::render::TermRenderer,
	util::sleep,
	world::{World, frame::FrameInput},
};
use std::{io, time::Instant};

pub mod keyboard;
pub mod raw;
pub mod render;

pub fn run(world: World, hide_title: bool) -> Result<()> {
	let mut world = world;

	let renderer = TermRenderer {
		hide_title,
		config: world.config().clone(),
		name: world.name(),
	};

	#[cfg(feature = "midi")]
	let mut player =
		crate::midi::MidiPlayer::new(world.config().midi.clone()).context("MIDI error!")?;

	let ctrl_c_rx = crate::util::setup_ctrl_c();

	let mut last_frame = Instant::now();

	while let Some(frame) = world.next_frame(&FrameInput {
		ext_in: keyboard::get_keys(world.config()),
	}) {
		if ctrl_c_rx.as_ref().is_some_and(|rx| rx.try_recv().is_ok()) {
			break;
		}

		renderer.render_frame(&frame);

		#[cfg(feature = "midi")]
		player.transmit(&frame.ext_out);

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

	Ok(())
}
