pub mod file_compiler;
pub mod frame;

mod state;
mod tick;

pub mod config;
use config::WorldConfig;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use std::{
	collections::BTreeMap,
	ops::{Deref, DerefMut},
};

use crate::{
	ant::{Ant, behavior::Behavior},
	util::dir::Direction,
	world::{
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
			..
		} = config;

		let start_pos = start_pos.get(height, width);

		let behaviors = &properties.behaviors;

		for (&id, behavior) in behaviors {
			if behavior.name.is_empty() {
				bail!("ant name must not be an empty string (found in ant with id = {id})")
			}
		}

		let ant = if let Some(root_id) = behaviors.keys().min() {
			Ant {
				dir: Direction::from(start_dir),
				behavior: *root_id,
				..Default::default()
			}
		} else {
			bail!("can't run a world with no ants defined")
		};

		state.ants.entry(0).or_default().insert(start_pos, ant);

		Ok(Self { properties, state })
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
