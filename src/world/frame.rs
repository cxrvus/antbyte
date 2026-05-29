use std::collections::BTreeMap;

use crate::{
	ant::Ant,
	util::vec2::Position,
	world::{
		World,
		config::{ColorMode, RenderMask},
		state::WorldStatus,
	},
};

#[derive(Debug, Default)]
pub struct FrameInput {
	pub ext_in: u8,
}

#[derive(Debug)]
pub struct FrameOutput {
	pub fg: BTreeMap<Position, u8>,
	pub bg: BTreeMap<Position, u8>,
	pub ms: Option<u32>,
	pub metadata: String, //todo: turn this into a map
	pub ext_out: Vec<u8>,
}

impl World {
	#[inline]
	/// like next_frame, but without input (defaulted to 0)
	pub fn next_frame_auto(&mut self) -> Option<FrameOutput> {
		self.next_frame(Default::default())
	}

	pub fn next_frame(&mut self, input: FrameInput) -> Option<FrameOutput> {
		let mut frame_ms = match self.config().fps {
			Some(0) => panic!(),
			Some(fps) => Some(1000 / fps),
			None => None,
		};

		match self.status {
			WorldStatus::Init => {
				self.status = WorldStatus::Active;
			}
			WorldStatus::Inactive => {
				if self.config().looping {
					// reset
					self.reset();
					self.status = WorldStatus::Active;
				} else {
					// stop
					return None;
				}
			}
			WorldStatus::Active => {
				self.ext_output.clear();
				self.ext_input = input.ext_in;

				let mut speed = self.config().speed.unwrap_or_default();

				if self.tick_count() == 0 && self.config().start_tick > 0 {
					// ignore external input and tick until start_tick is reached
					speed = self.config().start_tick;
					self.ext_input = 0;
				}

				for _tick in 0..speed {
					let active = self.tick();

					if !active {
						// current tick is last tick to be simulated
						frame_ms = self.config().sleep;
						self.status = WorldStatus::Inactive;
						break;
					}
				}
			}
		}

		let fg = self.get_render_values(&self.config().fg);
		let bg = self.get_render_values(&self.config().bg);

		Some(FrameOutput {
			fg,
			bg,
			ms: frame_ms,
			metadata: self.metadata_str(),
			ext_out: self.ext_output.clone(),
		})
	}

	fn get_render_values(&self, mask: &RenderMask) -> BTreeMap<Position, u8> {
		match mask {
			RenderMask::None => Default::default(),
			RenderMask::Cell => self.cells_to_map(),
			RenderMask::Dir => self.map_ants(|ant| ant.dir.value()),
			RenderMask::Id => self.map_ants(|ant| ant.behavior),
			RenderMask::BirthTick => self.map_ants(|ant| ant.birth_tick as u8),
			RenderMask::InputBits => self.map_ants(|ant| ant.last_input),
			RenderMask::Mem => self.map_ants(|ant| ant.memory),
		}
	}

	fn cells_to_map(&self) -> BTreeMap<Position, u8> {
		let width = self.config().width;

		let bg_entries = self
			.cells
			.entries
			.iter()
			.enumerate()
			.filter(|&(_, &value)| value != 0)
			.map(|(i, value)| (Position::from_index(i, width), self.adjusted_color(*value)));

		BTreeMap::from_iter(bg_entries)
	}

	fn map_ants(&self, func: impl Fn(&Ant) -> u8) -> BTreeMap<Position, u8> {
		self.ants
			.iter()
			.map(|(&pos, ant)| (pos, func(ant)))
			.collect()
	}

	// todo: implement using bg_filter
	fn adjusted_color(&self, color: u8) -> u8 {
		match self.config().color_mode {
			ColorMode::Binary => match color {
				0 => 0x0,
				_ => 0xf,
			},
			ColorMode::RGBI => color,
		}
	}
}
