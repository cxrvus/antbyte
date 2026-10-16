#![cfg(feature = "gif")]

use crate::{
	util::{clear_screen, print_title_short, vec2::Pos},
	world::{World, config::WorldConfig, frame::FrameOutput},
};

use anyhow::Result;
use gif::{Encoder, Frame, Repeat};
use std::{fs::File, path::Path};

const MAX_FRAMES: u32 = 0x400;
const MAX_PX: u16 = 0x200;

pub fn gif_export(world: &World, source: &Path) -> Result<()> {
	let path = {
		let mut path = source.to_path_buf();
		let mut file_name = path.file_name().unwrap().to_string_lossy().to_string();
		file_name.push_str(".gif");
		path.set_file_name(file_name);
		path
	};

	let WorldConfig { width, height, .. } = world.config();

	let max_dim = (*width).max(*height);

	#[rustfmt::skip]
	let scale = if max_dim <= MAX_PX { MAX_PX / max_dim } else { 1 }.max(1);

	let scaled_width = width * scale;
	let scaled_height = height * scale;

	let palette = world
		.config()
		.palette
		.to_vec()
		.iter()
		.flat_map(|&(r, g, b)| vec![r, g, b])
		.collect::<Vec<_>>();

	let mut image = File::create(&path)?;
	let mut encoder = Encoder::new(&mut image, scaled_width, scaled_height, palette.as_slice())?;
	encoder.set_repeat(Repeat::Infinite)?;

	let mut world = world.clone();

	for i in 0..MAX_FRAMES {
		clear_screen();
		print_title_short();
		println!("rendering frame {i} out of {MAX_FRAMES}...");
		if let Some(frame) = world.next_frame_auto() {
			render_frame(world.config(), &mut encoder, scale, &frame);
		} else {
			break;
		}
	}

	println!("done!\nGIF exported as '{}'", path.to_string_lossy());

	Ok(())
}

fn render_frame(
	config: &WorldConfig,
	encoder: &mut Encoder<&mut File>,
	scale: u16,
	frame: &FrameOutput,
) {
	let WorldConfig {
		width, height, fps, ..
	} = *config;

	let fps = fps.min(30);
	let delay = (100.0 / fps as f32).round() as u16;

	let scaled_width = width * scale;
	let scaled_height = height * scale;

	let mut scaled_pixels = Vec::with_capacity(scaled_width as usize * scaled_height as usize);

	for y in 0..height {
		let mut scaled_row = Vec::with_capacity(scaled_width as usize);

		for x in 0..width {
			let pos = Pos { x, y };
			let pixel = *frame.bg.get(&pos).unwrap_or(&0);
			let pixel = pixel & 0xf; // only take LSB half

			for _ in 0..scale {
				scaled_row.push(pixel);
			}
		}

		for _ in 0..scale {
			scaled_pixels.extend_from_slice(&scaled_row);
		}
	}

	let frame = Frame {
		width: scaled_width,
		height: scaled_height,
		buffer: scaled_pixels.into(),
		delay,
		..Frame::default()
	};

	encoder.write_frame(&frame).unwrap();
}
