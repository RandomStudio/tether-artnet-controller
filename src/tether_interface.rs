use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use log::debug;
use serde::{Deserialize, Serialize};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

#[derive(Serialize, Deserialize, Debug)]
pub struct TetherNotePayload {
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TetherControlChangePayload {
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

#[derive(Debug)]
pub enum TetherMidiMessage {
    /// Already-encoded payload
    Raw(Vec<u8>),
    NoteOn(TetherNotePayload),
    NoteOff(TetherNotePayload),
    ControlChange(TetherControlChangePayload),
}

pub fn start_tether_thread(tx: Sender<TetherMidiMessage>) -> JoinHandle<()> {
    let tether_agent = TetherAgentOptionsBuilder::new("ArtnetController")
        .build()
        .expect("failed to init Tether Agent");

    let input_midi_cc = PlugOptionsBuilder::create_input("controlChange")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let input_midi_notes = PlugOptionsBuilder::create_input("notesOn")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    spawn(move || loop {
        while let Some((topic, message)) = tether_agent.check_messages() {
            if input_midi_cc.matches(&topic) {
                debug!("MIDI CC");
                let m = rmp_serde::from_slice::<TetherControlChangePayload>(&message.payload())
                    .unwrap();
                tx.send(TetherMidiMessage::ControlChange(m))
                    .expect("failed to send")
            }
            if input_midi_notes.matches(&topic) {
                debug!("MIDI Note");
                let m = rmp_serde::from_slice::<TetherNotePayload>(&message.payload()).unwrap();
                tx.send(TetherMidiMessage::NoteOn(m))
                    .expect("failed to send")
            }
        }
        sleep(Duration::from_millis(1));
    })
}
