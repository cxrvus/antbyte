use std::collections::BTreeMap;

use crate::{
	ant::Ant,
	util::vec2::Pos,
	world::{
		World,
		config::{LAYER_CAP, RenderMask},
		state::WorldStatus,
	},
};

#[derive(Debug, Default, Clone)]
pub struct FrameInput {
	pub ext_in: u16,
}

#[derive(Debug, Clone)]
pub struct FrameOutput {
	pub fg: BTreeMap<Pos, u8>,
	pub bg: BTreeMap<Pos, u8>,
	pub ms: Option<u32>,
	pub metadata: String, //todo: turn this into a map
	pub ext_out: Vec<u16>,
}

impl World {
	#[inline]
	/// like next_frame, but without input (defaulted to 0)
	pub fn next_frame_auto(&mut self) -> Option<FrameOutput> {
		self.next_frame(&Default::default())
	}

	pub fn next_frame(&mut self, input: &FrameInput) -> Option<FrameOutput> {
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

		let bg = bg
			.iter()
			.map(|(&pos, &value)| (pos, self.config().bg_filter.apply(value)))
			.collect();

		Some(FrameOutput {
			fg,
			bg,
			ms: frame_ms,
			metadata: self.metadata_str(),
			ext_out: self.ext_output.clone(),
		})
	}

	fn get_render_values(&self, mask: &RenderMask) -> BTreeMap<Pos, u8> {
		match mask {
			RenderMask::None => Default::default(),
			RenderMask::Tile => self.tiles_to_map(),
			RenderMask::Layers => self.layer_occupations(),
			RenderMask::Dir => self.map_ants(|ant| ant.dir.value()),
			RenderMask::Id => self.map_ants(|ant| ant.behavior),
			RenderMask::BirthTick => self.map_ants(|ant| ant.birth_tick as u8),
			RenderMask::InputBits => self.map_ants(|ant| ant.last_input),
			RenderMask::Mem => self.map_ants(|ant| ant.memory),
		}
	}

	fn tiles_to_map(&self) -> BTreeMap<Pos, u8> {
		let width = self.config().width;

		let bg_entries = self
			.tiles
			.entries
			.iter()
			.enumerate()
			.filter(|&(_, &value)| value != 0)
			.map(|(i, &value)| (Pos::from_index(i, width), value));

		BTreeMap::from_iter(bg_entries)
	}

	fn layer_occupations(&self) -> BTreeMap<Pos, u8> {
		let mut map = BTreeMap::new();

		for (layer, ants) in self.ants.iter() {
			let new_value = 1u8 << layer;

			for (&pos, _) in ants.iter() {
				map.entry(pos)
					.and_modify(|old_value| *old_value |= new_value)
					.or_insert(new_value);
			}
		}

		map
	}

	fn map_ants(&self, func: impl Fn(&Ant) -> u8) -> BTreeMap<Pos, u8> {
		let mut map = BTreeMap::new();

		for i in (0..LAYER_CAP).rev() {
			let show_layer = (self.config().layer_filter >> i) & 1;

			if show_layer == 1
				&& let Some(layer) = self.ants.get(&i)
			{
				for (&pos, ant) in layer {
					map.entry(pos).or_insert(func(ant));
				}
			}
		}

		map
	}
}
