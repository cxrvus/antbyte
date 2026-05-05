pub mod file_compiler;
pub mod frame;

mod state;
mod tick;

pub mod config;
use config::{StartingPos, WorldConfig};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use std::{
	collections::BTreeMap,
	ops::{Deref, DerefMut},
};

use crate::{
	ant::{Ant, behavior::Behavior},
	util::{dir::Direction, vec2::Position},
	world::{config::ColorMode, state::WorldState},
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

pub struct World {
	properties: WorldProperties,
	pub state: WorldState,
}

impl World {
	pub fn new(properties: WorldProperties) -> Result<Self> {
		properties.config.validate()?;

		let config = properties.config.clone();

		let state = WorldState::new(&config);

		let WorldConfig {
			width,
			height,
			start_pos,
			start_dir,
			..
		} = config;

		let mut world = Self { properties, state };

		let half_width = (width - 1) / 2;
		let half_height = (height - 1) / 2;

		let start_pos = match start_pos {
			StartingPos::TopLeft => Position::ZERO,
			StartingPos::Top => Position {
				x: half_height,
				y: 0,
			},
			StartingPos::Left => Position {
				x: 0,
				y: half_height,
			},
			StartingPos::Center => Position {
				x: half_width,
				y: half_height,
			},
		};

		let ant = if let Some(root_id) = world.properties.behaviors.keys().min() {
			Ant {
				dir: Direction::new(start_dir),
				behavior: *root_id,
				..Default::default()
			}
		} else {
			bail!("can't run a world with no ants defined")
		};

		world.ants.insert(start_pos, ant);

		Ok(world)
	}

	#[inline]
	pub fn reset(&mut self) {
		*self = Self::new(self.properties.clone()).unwrap();
	}

	// TODO: render modes
	pub fn adjusted_color(&self, color: u8) -> u8 {
		match self.config().color_mode {
			ColorMode::Binary => match color {
				0 => 0x0,
				_ => 0xf,
			},
			ColorMode::RGBI => color,
		}
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
