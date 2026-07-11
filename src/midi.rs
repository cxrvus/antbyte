#![cfg(feature = "midi")]

use std::{
	collections::BTreeSet,
	io::{Write, stdin, stdout},
};

use anyhow::{Result, anyhow, bail};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::world::config::MidiConfig;

const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const VELOCITY: u8 = 0x7f;

pub struct MidiPlayer {
	config: MidiConfig,
	conn_out: Option<MidiOutputConnection>,
	held_notes: BTreeSet<(u8, u8)>,
}

impl MidiPlayer {
	pub fn new(config: MidiConfig) -> Result<Self> {
		let mut player = Self {
			config: config.clone(),
			conn_out: None,
			held_notes: BTreeSet::new(),
		};

		if !config.out_ch.is_empty() {
			player.conn_out = Some(connect_out()?);
		}

		Ok(player)
	}

	fn send_note(&mut self, ch: u8, note: u8, on: bool) {
		let on_off = if on { NOTE_ON } else { NOTE_OFF };
		let status = on_off | ch;
		let conn_out = self.conn_out.as_mut().unwrap();
		let _ = conn_out.send(&[status, note, VELOCITY]);
	}

	pub fn transmit(&mut self, values: &[u8]) {
		let offset = self.config.offset;

		if !self.config.out_ch.is_empty() {
			let new_notes: Vec<(u8, u8)> = values
				.iter()
				.map(|value| {
					let note = (value & 0b111111)
						.saturating_add(offset)
						.saturating_sub(1)
						.min(127);

					let slot = (value & 0b11000000) >> 6;
					let ch = match self.config.out_ch.get(&slot) {
						Some(0) | None => None,
						Some(ch) => Some(ch - 1),
					};

					(ch, note)
				})
				.filter_map(|(ch, note)| ch.map(|ch| (ch, note)))
				.collect();

			// send NOTE_ON for notes that are new
			for note in &new_notes {
				if !self.held_notes.contains(note) {
					self.send_note(note.0, note.1, true);
					self.held_notes.insert(*note);
				}
			}

			// send NOTE_OFF for held notes that are no longer present
			for note in self.held_notes.clone() {
				if !new_notes.contains(&note) {
					self.send_note(note.0, note.1, false);
					self.held_notes.remove(&note);
				}
			}
		}
	}
}

impl Drop for MidiPlayer {
	fn drop(&mut self) {
		// send NOTE_OFF for all held notes
		if !self.config.out_ch.is_empty() {
			for (ch, note) in self.held_notes.clone() {
				self.send_note(ch, note, false);
			}

			self.conn_out.take().unwrap().close();
		}
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
