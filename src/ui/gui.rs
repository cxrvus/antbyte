use bracket_lib::prelude::*;

use crate::world::{
	World,
	config::{RenderMask, WorldConfig},
};

const TILE_WIDTH: u32 = 8;
const TILE_HEIGHT: u32 = 8;
const MAX_ZOOM: f32 = 4.0;
const ZOOM_STEP: f32 = 1.0;
const PAN_STEP: f32 = 4.0;

struct State {
	zoom: f32,
	pan_x: f32,
	pan_y: f32,
	world: World,
}

impl State {
	fn handle_keys(&mut self, context: &mut BTerm) {
		use VirtualKeyCode as Key;

		let control = context.control;
		let mut input = INPUT.lock();

		input.for_each_message(|event| match event {
			BEvent::Character { .. } => {}
			BEvent::KeyboardInput { key, pressed, .. } => {
				if !pressed {
					return;
				}

				if control {
					match key {
						Key::Equals => self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM),
						Key::Minus => self.zoom = (self.zoom - ZOOM_STEP).max(1.0),
						Key::Q => std::process::exit(1),
						_ => {}
					}
				} else {
					match key {
						Key::Left => self.pan_x -= PAN_STEP,
						Key::Right => self.pan_x += PAN_STEP,
						Key::Up => self.pan_y -= PAN_STEP,
						Key::Down => self.pan_y += PAN_STEP,
						_ => {}
					}
				}

				context.set_scale(self.zoom, 0, 0);
				context.set_offset(self.pan_x, self.pan_y);
			}
			_ => {}
		});
	}
}

impl GameState for State {
	fn tick(&mut self, context: &mut BTerm) {
		self.handle_keys(context);

		if let Some(frame) = self.world.next_frame_auto() {
			// TODO: ext. input

			context.cls();

			if self.world.config().bg != RenderMask::None {
				for (&pos, &value) in &frame.bg {
					let bg_value = (value & 0x0f) * 17;
					let bg_color = RGB::from_u8(bg_value, bg_value, bg_value);
					let fg_color: RGB = BLACK.into();
					context.set(pos.x, pos.y, fg_color, bg_color, 0);
				}
			}

			// TODO: fix FG
			if self.world.config().fg != RenderMask::None {
				for (&pos, &fg_value) in &frame.fg {
					let bg_value = frame.bg.get(&pos).copied().unwrap_or(0) * 17;
					let bg_color = RGB::from_u8(bg_value, bg_value, bg_value);
					let fg_color: RGB = WHITE.into();

					context.set(pos.x, pos.y, fg_color, bg_color, fg_value);
				}
			}
		}
	}
}

pub fn run(world: World, fullscreen: bool) -> BError {
	let WorldConfig {
		width, height, fps, ..
	} = world.config();
	let (width, height) = (*width as u32, *height as u32);

	let assets_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

	let context = BTermBuilder::new()
		.with_advanced_input(true)
		.with_dimensions(width, height)
		.with_simple_console(width, height, "hex_font.png")
		.with_resource_path(assets_path)
		.with_font("hex_font.png", TILE_WIDTH, TILE_HEIGHT)
		.with_fitscreen(true)
		.with_fullscreen(fullscreen)
		.with_fps_cap(*fps as f32)
		.build()?;

	main_loop(
		context,
		State {
			zoom: 1.0,
			pan_x: 0.0,
			pan_y: 0.0,
			world,
		},
	)
}
