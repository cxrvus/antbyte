pub mod parser;

use rand::{Rng, SeedableRng, rngs::StdRng};

use std::{
	ops::{Deref, DerefMut},
	rc::Rc,
};

use crate::{
	ant::{
		Ant, BorderMode, StartingPos,
		archetype::Archetype,
		peripherals::{InputType, OutputType},
	},
	util::{matrix::Matrix, vec2::Vec2u},
};

type Cells = Matrix<u8>;

#[derive(Debug)]
pub struct WorldConfig {
	pub archetypes: Vec<Archetype>,
	pub width: usize,
	pub height: usize,
	pub border_mode: BorderMode,
	pub starting_pos: StartingPos,
	pub noise_seed: Option<u32>,
}

impl Default for WorldConfig {
	fn default() -> Self {
		Self {
			archetypes: vec![],
			width: 32,
			height: 32,
			border_mode: BorderMode::Collide,
			starting_pos: StartingPos::Center,
			noise_seed: None,
		}
	}
}

#[derive(Clone)]
pub struct WorldState {
	rng: StdRng,
	frame: usize,
	// todo: ant cache matrix
	pub cells: Cells,
	// todo: limit ant count
	pub ants: Vec<Ant>,
}

#[derive(Clone)]
pub struct World {
	config: Rc<WorldConfig>,
	pub state: WorldState,
}

impl World {
	pub fn new(config: WorldConfig) -> Self {
		let WorldConfig { width, height, .. } = config;

		let rng = if let Some(seed) = config.noise_seed {
			StdRng::seed_from_u64(seed as u64)
		} else {
			StdRng::from_seed(rand::random::<[u8; 32]>())
		};

		let mut state = WorldState {
			rng,
			frame: 0,
			cells: Matrix::new(width, height),
			ants: vec![],
		};

		if !config.archetypes.is_empty() {
			let starting_pos = match config.starting_pos {
				StartingPos::TopLeft => Vec2u::ZERO,
				StartingPos::Center => Vec2u {
					x: config.width / 2,
					y: config.height / 2,
				},
			};

			let mut ant = Ant::new(0);
			ant.pos = starting_pos;
			state.ants.push(ant);
		}

		Self {
			config: Rc::new(config),
			state,
		}
	}

	pub fn tick(&mut self) {
		// todo: optimize - remove cloning (here and in ant_tick)
		self.frame += 1;

		let mut world_frame = self.clone();

		for (i, ant) in self.ants.iter().enumerate() {
			let ant = world_frame.ant_tick(ant);
			world_frame.ants[i] = ant;
		}

		*self = world_frame;
	}

	pub fn ant_tick(&mut self, ant: &Ant) -> Ant {
		let world_image = self.clone();

		let Archetype {
			inputs,
			outputs,
			circuit,
			..
		} = world_image
			.get_archetype(ant.archetype)
			.expect("invalid Archetype ID");

		let mut condensed_input = 0u32;

		for input in inputs.iter() {
			use InputType::*;

			// getting the input value
			let input_value: u32 = match input.peripheral_type() {
				Time => ant.age % 0x100,
				Cell => (*self.cells.at(&ant.pos.sign()).unwrap()).into(),
				CellNext => ant
					.next_pos(self)
					.map(|pos| *self.cells.at(&pos.sign()).unwrap())
					.unwrap_or(0u8)
					.into(),
				Memory => ant.memory.current,
				Random => self.rng(),
				Ant => ant.get_target_ant(self).is_some().into(),
			};

			// condensing the input values into a single u32 value
			let bit_count = input.bit();
			let mask = 1u32.unbounded_shl(bit_count).wrapping_sub(1);
			let masked_input_value = input_value & mask;
			condensed_input <<= bit_count;
			condensed_input |= masked_input_value;
		}

		// calculating the output
		let mut condensed_output = circuit.tick(condensed_input);

		let mut ant = *ant;

		for output in outputs.iter() {
			use OutputType::*;

			// inflating the output bits into multiple u32 values
			let bit_count = output.bit();
			let mask = 1u32.unbounded_shl(bit_count).wrapping_sub(1);
			let value = condensed_output & mask;

			match output.peripheral_type() {
				Direction => {
					let moving = value & 1 == 1;
					let rotations = (value >> 1) as u8;
					ant.set_dir(ant.dir + rotations);

					if moving {
						ant.move_tick(self);
					}
				}
				CellWrite if value != 0 => self.cells.set_at(&ant.pos.sign(), 1),
				CellClear if value != 0 => self.cells.set_at(&ant.pos.sign(), 0),
				MemoryWrite => ant.memory.next = value,
				MemoryEnable => ant.memory.overwrite(),
				Spawn => {
					if let Some(pos) = ant.next_pos(self)
						&& value > 0
					{
						Ant::spawn(self, value - 1, pos);
					}
				}
				Kill => {
					if let Some(ant) = ant.get_target_ant(self) {
						ant.die();
					}
				}
				Die => ant.die(),
				_ => {}
			};

			condensed_output >>= bit_count;
		}

		ant
	}

	pub fn border_mode(&self) -> &BorderMode {
		&self.config.border_mode
	}

	pub fn get_archetype(&self, id: u32) -> Option<&Archetype> {
		self.config.archetypes.get(id as usize)
	}

	pub fn rng(&mut self) -> u32 {
		self.rng.random()
	}

	pub fn frame(&self) -> usize {
		self.frame
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
