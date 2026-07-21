#![cfg(feature = "midi")]

use std::{
	collections::BTreeMap,
	io::{Write, stdin, stdout},
};

use anyhow::{Result, anyhow, bail};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::world::config::MidiConfig;

const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const MAX_VELOCITY: u8 = 0x7f;
const OFFSET: u8 = 48; // C3

pub struct MidiPlayer {
	config: MidiConfig,
	conn_out: Option<MidiOutputConnection>,
	held_notes: BTreeMap<Note, u8>,
	// TODO: add velocity_offset
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Note {
	ch: u8,
	note: u8,
}

impl MidiPlayer {
	pub fn new(config: MidiConfig) -> Result<Self> {
		let mut player = Self {
			config: config.clone(),
			conn_out: None,
			held_notes: Default::default(),
		};

		if !config.out_ch.is_empty() {
			player.conn_out = Some(connect_out()?);
		}

		Ok(player)
	}

	pub fn close(&mut self) {
		println!("\nClosing MIDI connections...");

		// send NOTE_OFF for all held notes
		if !self.config.out_ch.is_empty() {
			for note in self.held_notes.clone().keys() {
				self.send_note(note, None);
			}

			self.conn_out.take().map(|c| c.close());
		}
	}

	fn send_note(&mut self, note: &Note, vel: Option<u8>) {
		let (on_off, vel) = match vel {
			Some(vel) => (NOTE_ON, vel),
			None => (NOTE_OFF, 0),
		};

		let status = on_off | note.ch;
		let conn_out = self.conn_out.as_mut().unwrap();
		let _ = conn_out.send(&[status, note.note, vel]);
	}

	fn parse_note(&self, value: u16) -> Option<(Note, u8)> {
		let inv_vel = ((value >> 8) & 0b1111) << 3;
		let vel = MAX_VELOCITY.saturating_sub(inv_vel as u8);
		let note = (value & 0b111111) as u8;

		let slot = ((value >> 6) & 0b11) as u8;

		let ch = match self.config.out_ch.get(&slot) {
			Some(0) | None => return None,
			Some(ch) => ch - 1,
		};

		if note & vel == 0 {
			return None;
		}

		let offset = self.config.offset.get(&slot).unwrap_or(&OFFSET);
		let note = note.saturating_sub(1).saturating_add(*offset).min(127);

		Some((Note { ch, note }, vel))
	}

	pub fn transmit(&mut self, values: &[u16]) {
		if !self.config.out_ch.is_empty() {
			let prev_notes = self.held_notes.clone();
			let mut new_notes = BTreeMap::<Note, u8>::new();

			for &value in values {
				if let Some((note, new_vel)) = self.parse_note(value) {
					new_notes
						.entry(note)
						.and_modify(|old_vel| *old_vel = (*old_vel).max(new_vel))
						.or_insert(new_vel);
				}
			}

			// send NOTE_ON for notes that are new
			for (note, &vel) in &new_notes {
				if !prev_notes.contains_key(note) {
					self.send_note(note, Some(vel));
					self.held_notes.insert(note.clone(), vel);
				}
			}

			// send NOTE_OFF for held notes that are no longer present
			for note in prev_notes.keys() {
				if !new_notes.contains_key(note) {
					self.send_note(note, None);
					self.held_notes.remove(note);
				}
			}
		}
	}
}

impl Drop for MidiPlayer {
	fn drop(&mut self) {
		self.close();
	}
}

fn connect_out() -> Result<MidiOutputConnection> {
	let midi_out = MidiOutput::new("ANTBYTE MIDI")?;
	let out_ports = midi_out.ports();

	let out_port: &MidiOutputPort = match out_ports.len() {
		0 => bail!("no output port found"),
		1 => {
			println!(
				"Choosing the only available output port: {}",
				midi_out.port_name(&out_ports[0]).unwrap()
			);
			&out_ports[0]
		}
		2 => {
			println!(
				"Choosing the second output port: {}",
				midi_out.port_name(&out_ports[1]).unwrap()
			);
			&out_ports[1]
		}
		_ => {
			println!("\nAvailable output ports:");
			for (i, p) in out_ports.iter().enumerate() {
				println!("{}: {}", i, midi_out.port_name(p).unwrap());
			}
			println!("Please select output port:");
			stdout().flush()?;
			let mut input = String::new();
			stdin().read_line(&mut input)?;
			out_ports
				.get(input.trim().parse::<usize>()?)
				.ok_or(anyhow!("invalid output port selected"))?
		}
	};

	println!("Opening MIDI OUT connection...");

	midi_out
		.connect(out_port, "ANTBYTE MIDI (OUT)")
		.map_err(|e| anyhow!("{e}"))
}
