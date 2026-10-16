#![cfg(feature = "gui")]

use VirtualKeyCode as Key;
use bracket_lib::prelude::*;
use std::collections::{BTreeMap, HashSet};

use crate::{
	render::{chars::key_to_char, chars_to_input},
	util::{print_stationary, sleep, vec2::Pos},
	world::{
		World,
		config::{RenderMask, WorldConfig},
		frame::FrameInput,
	},
};

const TILE_WIDTH: u32 = 8;
const TILE_HEIGHT: u32 = 8;
const MAX_ZOOM: f32 = 8.0;
const MIN_ZOOM: f32 = 0.5;
const ZOOM_STEP: f32 = 0.5;
const PAN_STEP: f32 = 4.0;
const METADATA_LINES: usize = 3;

struct State {
	zoom: f32,
	pan_x: f32,
	pan_y: f32,
	world: World,
	metadata: String,
	held_keys: HashSet<Key>,
	end: bool,
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
						Key::Minus => self.zoom = (self.zoom - ZOOM_STEP).max(MIN_ZOOM),
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
	fn print_metadata(&self) {
		if !self.metadata.is_empty() {
			debug_assert_eq!(self.metadata.lines().count(), METADATA_LINES);
			print_stationary(&self.metadata);
		}
	}
}

impl GameState for State {
	fn tick(&mut self, context: &mut BTerm) {
		let input_chars = self.handle_keys(context);
		let input_value = chars_to_input(self.world.config().keys.as_deref(), &input_chars);
		let input = FrameInput {
			ext_in: input_value,
		};

		if self.world.config().stepped && input_chars.is_empty() {
			return;
		}

		if self.end {
			// context.quit();
		} else if let Some(frame) = self.world.next_frame(&input) {
			self.print_metadata();
			cls(context, &self.world);

			let mut tiles: BTreeMap<Pos, (u8, u8)> = frame
				.bg
				.iter()
				.map(|(&pos, &value)| (pos, (value, 0u8)))
				.collect();

			let fg_mode = self.world.config().fg;

			for (&pos, &fg_value) in &frame.fg {
				let fg_value = map_fg_value(&fg_mode, fg_value);

				tiles
					.entry(pos)
					.and_modify(|(_, value)| *value = fg_value)
					.or_insert((0, fg_value));
			}

			for (&pos, &(bg_value, fg_value)) in &tiles {
				let Pos { x, y } = pos;
				let (r, g, b) = get_color(&self.world, bg_value);

				let bg_color = RGB::from_u8(r, g, b);
				let fg_color = RGB::from_u8(0xff - r, 0xff - g, 0xff - b);
				let fg_value = fg_value.wrapping_add(0x20); // needs to be offset because space (CLS) is at 0x20

				context.set(x, y, fg_color, bg_color, fg_value);
			}

			if let Some(ms) = self.world.config().loop_delay
				&& frame.after_reset
			{
				sleep(ms);
			} else {
				self.metadata = frame.metadata;
			}
		} else {
			println!("\nSimulation terminated.");
			self.end = true;
		}
	}
}

pub fn run(world: World, fullscreen: bool, simple: bool) -> BError {
	println!("{}", "\n".repeat(METADATA_LINES));

	let WorldConfig {
		width, height, fps, ..
	} = world.config();
	let (width, height) = (*width as u32, *height as u32);

	let assets_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

	let title = match world.name() {
		Some(name) => format!("{name}   -   AntByte"),
		None => "AntByte".into(),
	};

	let window_scale = match height.max(width) {
		0..=128 => 1.0,
		129..196 => 0.75,
		_ => 0.5,
	};

	let window_width = (width as f32 * window_scale).ceil() as u32;
	let window_height = (height as f32 * window_scale).ceil() as u32;

	let font = choose_font(&world.config().fg);

	let mut context = BTermBuilder::new()
		.with_title(title)
		.with_advanced_input(true)
		.with_dimensions(window_width, window_height)
		.with_simple_console(width, height, font.clone())
		.with_resource_path(assets_path)
		.with_font(font, TILE_WIDTH, TILE_HEIGHT)
		.with_fitscreen(true)
		.with_fullscreen(fullscreen)
		.with_fps_cap(*fps as f32)
		.build()?;

	if !simple {
		context.with_post_scanlines(false);
	}

	cls(&mut context, &world);

	main_loop(
		context,
		State {
			zoom: 1.0,
			pan_x: 0.0,
			pan_y: 0.0,
			world,
			metadata: "".into(),
			held_keys: HashSet::new(),
			end: false,
		},
	)
}

#[inline]
fn cls(context: &mut BTerm, world: &World) {
	context.cls();
	context.cls_bg(get_color(world, 0));
}

#[inline]
fn get_color(world: &World, value: u8) -> (u8, u8, u8) {
	*world.config().palette.get(&value).unwrap_or(&BLACK)
}

fn map_fg_value(mode: &RenderMask, value: u8) -> u8 {
	match mode {
		RenderMask::Dir => value + 1,
		RenderMask::Id if value == 0 => 0xff,
		_ => value,
	}
}

fn choose_font(mode: &RenderMask) -> String {
	match mode {
		RenderMask::Dir => "dir_font.png",
		_ => "hex_font.png",
	}
	.into()
}
