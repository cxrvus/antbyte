use std::{
	collections::BTreeMap,
	ops::{Deref, DerefMut},
};

use crate::{
	ant::Ant,
	util::{grid::Grid, vec2::Pos},
	world::config::WorldConfig,
};
use rand::{RngExt, SeedableRng, rngs::SmallRng};

pub type Tile = u8;

pub type Tiles = Grid<Tile>;
pub type Ants = BTreeMap<Pos, Ant>;

#[derive(Clone, Default, Debug)]
pub enum WorldStatus {
	#[default]
	Init,
	Active,
	Inactive,
}

#[derive(Clone, Default, Debug)]
pub struct WorldState {
	rng: Option<SmallRng>,
	pub(super) tick_count: u32,
	pub(super) status: WorldStatus,
	pub tiles: Tiles,
	pub tile_decays: BTreeMap<Pos, u16>,
	pub ants: Layers,
	pub signal_in: u8,
	pub signal_out: u8,
	pub ext_input: u16,
	pub ext_output: Vec<u16>,
}

impl WorldState {
	pub(super) fn new(config: &WorldConfig) -> Self {
		let tiles = Grid::new(config.width, config.height);

		let rng = if let Some(seed) = config.seed {
			Some(SmallRng::seed_from_u64(seed as u64))
		} else {
			Some(SmallRng::from_seed(rand::random::<[u8; 32]>()))
		};

		Self {
			tiles,
			rng,
			..Default::default()
		}
	}

	#[inline]
	pub fn tick_count(&self) -> u32 {
		self.tick_count
	}

	#[inline]
	pub(super) fn rng(&mut self) -> u8 {
		self.rng.as_mut().expect("rng must be Some").random()
	}

	pub(super) fn tile_decay(&mut self) {
		let current_tick = self.tick_count as u16;

		for (pos, expiration) in self.tile_decays.clone() {
			if current_tick == expiration {
				self.tiles.set(pos, 0);
				self.tile_decays.remove(&pos);
			}
		}
	}

	// formatting ...

	#[inline]
	pub fn tick_str(&self) -> String {
		format!("t: {:0>8}", self.tick_count())
	}

	pub fn ext_out_str(&self) -> Option<String> {
		const MAX_LEN: usize = 16;

		if self.ext_output.is_empty() {
			None
		} else {
			let mut ext_out_str = self
				.ext_output
				.iter()
				.take(MAX_LEN)
				.map(|x| format!("{x:02x}"))
				.collect::<Vec<_>>()
				.join(", ");

			if self.ext_output.len() > MAX_LEN {
				ext_out_str += ", ...";
			}

			Some(ext_out_str)
		}
	}

	pub fn metadata_str(&self) -> String {
		let mut metadata_str = self.tick_str();

		metadata_str += &format!(" | A: {:04}", self.ants.ant_count());

		if self.signal_in != 0 {
			metadata_str += &format!("\nS: {:08b}", self.signal_in);
		}
		if self.ext_input != 0 {
			metadata_str += &format!("\nX: {:08b}", self.ext_input);
		}
		if let Some(ext_out_str) = self.ext_out_str() {
			metadata_str += &format!("\nY: {}", ext_out_str);
		}

		metadata_str
	}
}

#[derive(Debug, Clone, Default)]
pub struct Layers(LayerContainer);
type LayerContainer = BTreeMap<u8, Ants>;

impl Layers {
	pub fn layer_mut(&mut self, layer: u8) -> &mut Ants {
		self.get_mut(&layer).unwrap()
	}

	pub fn ant_count(&self) -> usize {
		self.values().map(|layer| layer.len()).sum()
	}
}

impl Deref for Layers {
	type Target = LayerContainer;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Layers {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
