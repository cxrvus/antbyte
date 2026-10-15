#![cfg(feature = "gui")]

use VirtualKeyCode as Key;
use bracket_lib::prelude::*;
use std::collections::HashSet;

use crate::{
	render::{chars::key_to_char, chars_to_input},
	util::{print_stationary, sleep},
	world::{
		World,
		config::{RenderMask, WorldConfig},
		frame::FrameInput,
	},
};

const TILE_WIDTH: u32 = 8;
const TILE_HEIGHT: u32 = 8;
const MAX_ZOOM: f32 = 4.0;
const ZOOM_STEP: f32 = 1.0;
const PAN_STEP: f32 = 4.0;
const METADATA_LINES: usize = 3;

struct State {
	zoom: f32,
	pan_x: f32,
	pan_y: f32,
	world: World,
	metadata: String,
	held_keys: HashSet<Key>,
	quit: bool,
	sleep: Option<u32>,
}

impl State {
	fn handle_keys(&mut self, context: &mut BTerm) -> String {
		let control = context.control;
		let mut input = INPUT.lock();

		input.for_each_message(|event| match event {
			BEvent::CloseRequested => context.quit(),
			BEvent::Focused { focused: false } => self.held_keys.clear(),
			BEvent::KeyboardInput { key, pressed, .. } => {
				if pressed && control {
					match key {
						Key::Equals => self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM),
						Key::Minus => self.zoom = (self.zoom - ZOOM_STEP).max(1.0),
						Key::Q => context.quit(),
						_ => {}
					}
				} else {
					if pressed {
						match key {
							Key::Left => self.pan_x -= PAN_STEP,
							Key::Right => self.pan_x += PAN_STEP,
							Key::Up => self.pan_y -= PAN_STEP,
							Key::Down => self.pan_y += PAN_STEP,

							_ if key_to_char(&key).is_some() => {
								self.held_keys.insert(key);
							}

							_ => {}
						}
					} else {
						self.held_keys.remove(&key);
					}
				}

				context.set_scale(self.zoom, 0, 0);
				context.set_offset(self.pan_x, self.pan_y);
			}
			_ => {}
		});

		self.held_keys.iter().filter_map(key_to_char).collect()
	}

	#[inline]
	fn try_sleep(&mut self) {
		if let Some(ms) = self.sleep {
			sleep(ms);
			self.sleep = None;
		}
	}

	#[inline]
	fn print_metadata(&self) {
		if !self.metadata.is_empty() {
			debug_assert_eq!(self.metadata.lines().count(), METADATA_LINES);
			print_stationary(&self.metadata);
		}
	}
}

impl GameState for State {
	fn tick(&mut self, context: &mut BTerm) {
		self.print_metadata();
		let mut schedule_sleep = false;

		let input_chars = self.handle_keys(context);
		let input_value = chars_to_input(self.world.config().keys.as_deref(), &input_chars);
		let input = FrameInput {
			ext_in: input_value,
		};

		if self.world.config().stepped && input_chars.is_empty() {
			return;
		}

		if self.quit {
			context.quit();
		} else if let Some(frame) = self.world.next_frame(&input) {
			context.cls();

			for (&pos, &bg_value) in &frame.bg {
				let bg_value = match self.world.config().bg {
					RenderMask::None => 0,
					_ => bg_value,
				};
				let bg_value = (bg_value & 0x0f) * 17;

				let bg_color = RGB::from_u8(bg_value, bg_value, bg_value);
				let neg_bg_value = bg_value.wrapping_neg();
				let fg_color = RGB::from_u8(neg_bg_value, neg_bg_value, neg_bg_value);

				let fg_value = *frame.fg.get(&pos).unwrap_or(&0);
				let fg_value = match self.world.config().fg {
					RenderMask::None => 0,
					_ => fg_value,
				};
				// font texture needs to be offset because space (CLS) is at 0x20
				let fg_value = fg_value.wrapping_add(0x20);

				context.set(pos.x, pos.y, fg_color, bg_color, fg_value);
			}

			if frame.after_reset {
				schedule_sleep = true;
			} else {
				self.metadata = frame.metadata;
			}
		} else {
			self.quit = true;
			schedule_sleep = true;
		}

		self.try_sleep();

		if schedule_sleep {
			self.sleep = self.world.config().sleep;
		}
	}
}

pub fn run(world: World, fullscreen: bool) -> BError {
	let WorldConfig {
		width, height, fps, ..
	} = world.config();
	let (width, height) = (*width as u32, *height as u32);

	let assets_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

	let title = match world.name() {
		Some(name) => format!("{name}   -   AntByte"),
		None => "AntByte".into(),
	};

	println!("{}", "\n".repeat(METADATA_LINES));

	let mut context = BTermBuilder::new()
		.with_title(title)
		.with_advanced_input(true)
		.with_dimensions(width, height)
		.with_simple_console(width, height, "hex_font.png")
		.with_resource_path(assets_path)
		.with_font("hex_font.png", TILE_WIDTH, TILE_HEIGHT)
		.with_fitscreen(true)
		.with_fullscreen(fullscreen)
		.with_fps_cap(*fps as f32)
		.build()?;

	context.cls();

	main_loop(
		context,
		State {
			zoom: 1.0,
			pan_x: 0.0,
			pan_y: 0.0,
			world,
			metadata: "".into(),
			held_keys: HashSet::new(),
			quit: false,
			sleep: None,
		},
	)
}
