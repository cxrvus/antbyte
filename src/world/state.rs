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

pub type Cell = u8;

pub type Cells = Grid<Cell>;
pub type Ants = BTreeMap<Pos, Ant>;

#[derive(Clone, Default)]
pub enum WorldStatus {
	#[default]
	Init,
	Active,
	Inactive,
}

#[derive(Clone, Default)]
pub struct WorldState {
	rng: Option<SmallRng>,
	pub(super) tick_count: u32,
	pub(super) status: WorldStatus,
	pub cells: Cells,
	pub cell_decays: BTreeMap<Pos, u16>,
	pub ants: Layers,
	pub signal_in: u8,
	pub signal_out: u8,
	pub ext_input: u8,
	pub ext_output: Vec<u8>,
}

impl WorldState {
	pub(super) fn new(config: &WorldConfig) -> Self {
		let cells = Grid::new(config.width, config.height);

		let rng = if let Some(seed) = config.seed {
			Some(SmallRng::seed_from_u64(seed as u64))
		} else {
			Some(SmallRng::from_seed(rand::random::<[u8; 32]>()))
		};

		Self {
			cells,
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

	pub(super) fn cell_decay(&mut self) {
		let current_tick = self.tick_count as u16;

		for (pos, expiration) in self.cell_decays.clone() {
			if current_tick == expiration {
				self.cells.set(pos, 0);
				self.cell_decays.remove(&pos);
			}
		}
	}

	// formatting ...

	#[inline]
	pub fn tick_str(&self) -> String {
		format!("T: {:0>8}", self.tick_count())
	}

	pub fn ext_out_str(&self) -> String {
		let ext_out_str = self
			.ext_output
			.iter()
			.map(|x| format!("{x:02x}"))
			.collect::<Vec<_>>()
			.join(", ");

		if ext_out_str.is_empty() {
			"--".into()
		} else {
			ext_out_str
		}
	}

	pub fn metadata_str(&self) -> String {
		let tick_str = self.tick_str();
		let ext_out_str = self.ext_out_str();

		format!("{tick_str}\nK: {:02x}\nX: {ext_out_str}\n", self.ext_input)
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
