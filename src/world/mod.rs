pub mod cache;
pub mod config;
pub mod file_compiler;
pub mod frame;
pub mod palette;

use config::WorldConfig;

mod state;
mod tick;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use std::{
	collections::BTreeMap,
	ops::{Deref, DerefMut},
};

use crate::{
	ant::{Ant, behavior::Behavior},
	util::rotation::Rotation,
	world::{
		cache::WorldCache,
		config::BorderMode,
		state::{WorldState, WorldStatus},
	},
};

#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct WorldProperties {
	pub name: Option<String>,
	#[serde(rename = "ants")]
	pub behaviors: BTreeMap<u8, Behavior>,
	#[serde(rename = "cfg")]
	pub config: WorldConfig,
}

#[derive(Clone)]
pub struct World {
	properties: WorldProperties,
	pub state: WorldState,
	pub cache: WorldCache,
}

impl World {
	pub fn new(properties: WorldProperties) -> Result<Self> {
		properties.config.validate()?;

		let config = properties.config.clone();

		let mut state = WorldState::new(&config);

		if config.start_tick > 0 {
			state.status = WorldStatus::Active;
		}

		let WorldConfig {
			width,
			height,
			start_pos,
			start_dir,
			euclid,
			..
		} = config;

		let start_pos = start_pos.get(height, width);

		let behaviors = &properties.behaviors;

		for (&id, behavior) in behaviors {
			if behavior.name.is_empty() {
				bail!("ant name must not be an empty string (found in ant with id = {id})")
			}
		}

		// TODO: create constructor for ant
		let rotation = Rotation::from(start_dir);

		let ant = if let Some(root_id) = behaviors.keys().min() {
			Ant {
				rotation,
				birth_rot: rotation,
				behavior: *root_id,
				..Default::default()
			}
		} else {
			bail!("can't run a world with no ants defined")
		};

		state.ants.entry(0).or_default().insert(start_pos, ant);

		let cache = WorldCache::new(euclid);

		Ok(Self {
			properties,
			state,
			cache,
		})
	}

	#[inline]
	pub fn reset(&mut self) {
		*self = Self::new(self.properties.clone()).unwrap();
	}

	#[inline]
	pub fn name(&self) -> Option<String> {
		self.properties.name.clone()
	}

	#[inline]
	pub fn config(&self) -> &WorldConfig {
		&self.properties.config
	}

	#[inline]
	pub fn config_mut(&mut self) -> &mut WorldConfig {
		&mut self.properties.config
	}

	#[inline]
	fn get_behavior(&self, id: u8) -> Option<&Behavior> {
		self.properties.behaviors.get(&id)
	}

	fn border_mode(&self, layer: u8) -> BorderMode {
		self.config()
			.border
			.get(&layer)
			.unwrap_or(&self.config().border[&0])
			.clone()
	}

	fn ext_out_str(&self) -> String {
		const MAX_LEN: usize = 16;

		if self.ext_output.is_empty() {
			"--".into()
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

			ext_out_str
		}
	}

	pub fn metadata_str(&self) -> String {
		let WorldConfig {
			speed,
			fps,
			stepped,
			..
		} = self.config();

		let mut metadata_str = String::new();

		let stepped = match stepped {
			true => "(stepped);",
			false => "",
		};

		metadata_str += &format!("fps: {fps:02}; tpf: {speed:03}; {stepped}\n");
		metadata_str += &self.tick_str();
		metadata_str += &format!(" | A: {:04}", self.ants.ant_count());
		metadata_str += &format!("\nS: {:08b}", self.signal_in);
		metadata_str += &format!(" | I: {:08b}", self.ext_input);
		metadata_str += &format!("\nO: {}", self.ext_out_str());

		metadata_str
	}
}

impl Deref for World {
	type Target = WorldState;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}

impl DerefMut for World {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.state
	}
}
