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
const VELOCITY: u8 = 0x64;

pub struct MidiPlayer {
	config: MidiConfig,
	conn_out: Option<MidiOutputConnection>,
	held_notes: BTreeSet<u8>,
}

impl MidiPlayer {
	pub fn new(config: MidiConfig) -> Result<Self> {
		let mut player = Self {
			config: config.clone(),
			conn_out: None,
			held_notes: BTreeSet::new(),
		};

		if config.out_ch != 0 {
			player.conn_out = Some(connect_out()?);
		}

		Ok(player)
	}

	pub fn transmit(&mut self, new_notes: &[u8]) {
		if let Some(conn_out) = self.conn_out.as_mut() {
			let offset = self.config.offset;

			let channel = match self.config.out_ch {
				ch @ 1..=16 => ch - 1,
				_ => panic!("channel is out of range"),
			};

			let new_notes: Vec<u8> = new_notes
				.iter()
				.map(|n| (n & 0x7f).saturating_add(offset).saturating_sub(1).min(127))
				.collect();

			// send NOTE_ON for notes that are new
			for &note in &new_notes {
				if !self.held_notes.contains(&note) {
					let status = NOTE_ON | channel;
					let _ = conn_out.send(&[status, note, VELOCITY]);
					self.held_notes.insert(note);
				}
			}

			// send NOTE_OFF for held notes that are no longer present
			for held_note in self.held_notes.clone() {
				if !new_notes.contains(&held_note) {
					let status = NOTE_OFF | channel;
					let _ = conn_out.send(&[status, held_note, VELOCITY]);
					self.held_notes.remove(&held_note);
				}
			}
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
