use std::collections::BTreeMap;

use crate::{
	util::vec2::Position,
	world::{World, state::WorldStatus},
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

		// todo: implement render modes
		let fg = self
			.ants
			.iter()
			.map(|(&pos, ant)| (pos, ant.dir.value()))
			.collect();

		// TODO: implement cells as BTreeMap
		let bg = self
			.cells
			.entries
			.iter()
			.enumerate()
			.filter(|(_i, cell)| cell.value != 0)
			.map(|(i, cell)| {
				let value = cell.value;
				let pos = Position {
					x: (i % (self.config().width as usize)) as u16,
					y: (i / (self.config().height as usize)) as u16,
				};
				(pos, value)
			})
			.collect();

		Some(FrameOutput {
			fg,
			bg,
			ms: frame_ms,
			ext_out: self.ext_output.clone(),
		})
	}
}
