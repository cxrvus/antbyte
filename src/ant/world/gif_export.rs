#![cfg(feature = "gif")]

use crate::{
	ant::world::{World, WorldConfig},
	cli::{clear_screen, print_title_short},
};
use anyhow::{Result, bail};
use gif::{Encoder, Frame, Repeat};
use std::{fs::File, path::PathBuf};

const MAX_FRAMES: u32 = 0x100;
const MAX_PX: usize = 0x200;

impl World {
	pub fn gif_export(self, opt_path: Option<PathBuf>) -> Result<()> {
		let path = match opt_path {
			Some(path) => path,
			None => {
				// todo: add default path generation
				bail!("please provide a target path for the exported GIF")
			}
		};

		let WorldConfig { width, height, .. } = self.config();

		let max_dim = (*width).max(*height);
		#[rustfmt::skip]
		let scale = if max_dim <= MAX_PX { MAX_PX / max_dim } else { 1 }.max(1);
		let scaled_width = (width * scale) as u16;
		let scaled_height = (height * scale) as u16;

		let mut image = File::create(&path)?;
		let mut encoder = Encoder::new(&mut image, scaled_width, scaled_height, &PALETTE)?;
		encoder.set_repeat(Repeat::Infinite)?; // idea: set repeat corresponding to loop setting

		let mut world = self;

		for i in 0..MAX_FRAMES {
			clear_screen();
			print_title_short();
			println!("rendering frame {i} out of {MAX_FRAMES}...");
			world.gif_render(&mut encoder, scale);

			if !world.frame_tick() {
				break;
			}
		}

		println!("rendering final frame...");
		world.gif_render(&mut encoder, scale);
		println!("done!\nGif exported to {}", path.to_string_lossy());

		Ok(())
	}

	fn gif_render(&mut self, encoder: &mut Encoder<&mut File>, scale: usize) {
		let WorldConfig {
			width, height, fps, ..
		} = self.config();

		let scaled_width = width * scale;
		let scaled_height = height * scale;

		let mut scaled_pixels = Vec::with_capacity(scaled_width * scaled_height);

		for y in 0..*height {
			let mut scaled_row = Vec::with_capacity(scaled_width);
			for x in 0..*width {
				let pixel = self.cells.values[y * width + x];
				for _ in 0..scale {
					scaled_row.push(pixel);
				}
			}

			for _ in 0..scale {
				scaled_pixels.extend_from_slice(&scaled_row);
			}
		}

		let fps = fps.unwrap_or(30).clamp(1, 30);
		let delay = (100.0 / fps as f32).round() as u16;

		let frame = Frame {
			width: scaled_width as u16,
			height: scaled_height as u16,
			buffer: scaled_pixels.into(),
			delay,
			..Frame::default()
		};

		encoder.write_frame(&frame).unwrap();
	}
}

const PALETTE: [u8; 0x10 * 3] = [
	0x00, 0x00, 0x00, // 0: Black
	0x80, 0x00, 0x00, // 1: Dark Red
	0x00, 0x80, 0x00, // 2: Dark Green
	0x80, 0x80, 0x00, // 3: Dark Yellow/Brown
	0x00, 0x00, 0x80, // 4: Dark Blue
	0x80, 0x00, 0x80, // 5: Dark Magenta
	0x00, 0x80, 0x80, // 6: Dark Cyan
	0xC0, 0xC0, 0xC0, // 7: Light Gray
	0x80, 0x80, 0x80, // 8: Dark Gray
	0xFF, 0x00, 0x00, // 9: Bright Red
	0x00, 0xFF, 0x00, // 10: Bright Green
	0xFF, 0xFF, 0x00, // 11: Bright Yellow
	0x00, 0x00, 0xFF, // 12: Bright Blue
	0xFF, 0x00, 0xFF, // 13: Bright Magenta
	0x00, 0xFF, 0xFF, // 14: Bright Cyan
	0xFF, 0xFF, 0xFF, // 15: White
];
