use std::{
    sync::mpsc::Sender,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMacroMessage {
    /// If no fixture specified, assume all
    pub fixture_label: Option<String>,
    pub macro_label: String,
    pub value: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteAnimationMessage {
    /// If no fixture specified, assume all
    pub fixture_label: Option<String>,
    pub macro_label: String,
    /// Start value will be "whatever the current value is";
    /// so `target_value` is the End value
    pub target_value: u8,
    /// Animation duration in ms
    pub duration: u64,
}
#[derive(Debug)]
pub enum TetherMidiMessage {
    /// Already-encoded payload
    Raw(Vec<u8>),
    NoteOn(TetherNotePayload),
    NoteOff(TetherNotePayload),
    ControlChange(TetherControlChangePayload),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSceneMessage {
    pub scene_label: String,
}

pub enum RemoteControlMessage {
    Midi(TetherMidiMessage),
    MacroDirect(RemoteMacroMessage),
    MacroAnimation(RemoteAnimationMessage),
    SceneAnimation(RemoteSceneMessage),
}

pub fn start_tether_thread(tx: Sender<RemoteControlMessage>) -> JoinHandle<()> {
    let tether_agent = TetherAgentOptionsBuilder::new("ArtnetController")
        .build()
        .expect("failed to init Tether Agent");

    let input_midi_cc = PlugOptionsBuilder::create_input("controlChange")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let input_midi_notes = PlugOptionsBuilder::create_input("notesOn")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let input_macros = PlugOptionsBuilder::create_input("macros")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let input_animations = PlugOptionsBuilder::create_input("animations")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let input_scenes = PlugOptionsBuilder::create_input("scenes")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    spawn(move || loop {
        while let Some((topic, message)) = tether_agent.check_messages() {
            if input_midi_cc.matches(&topic) {
                debug!("MIDI CC");
                let m = rmp_serde::from_slice::<TetherControlChangePayload>(&message.payload())
                    .unwrap();
                tx.send(RemoteControlMessage::Midi(
                    TetherMidiMessage::ControlChange(m),
                ))
                .expect("failed to send from Tether Interface thread")
            }
            if input_midi_notes.matches(&topic) {
                debug!("MIDI Note");
                let m = rmp_serde::from_slice::<TetherNotePayload>(&message.payload()).unwrap();
                tx.send(RemoteControlMessage::Midi(TetherMidiMessage::NoteOn(m)))
                    .expect("failed to send from Tether Interface thread")
            }
            if input_macros.matches(&topic) {
                debug!("Macro (direct) control message");
                let m = rmp_serde::from_slice::<RemoteMacroMessage>(&message.payload()).unwrap();
                tx.send(RemoteControlMessage::MacroDirect(m))
                    .expect("failed to send from Tether Interface thread");
            }
            if input_animations.matches(&topic) {
                debug!("Macro Animation control message");
                let m =
                    rmp_serde::from_slice::<RemoteAnimationMessage>(&message.payload()).unwrap();
                tx.send(RemoteControlMessage::MacroAnimation(m))
                    .expect("failed to send from Tether Interface thread");
            }
            if input_scenes.matches(&topic) {
                debug!("Remote Scene message");
                let m = rmp_serde::from_slice::<RemoteSceneMessage>(&message.payload()).unwrap();
                tx.send(RemoteControlMessage::SceneAnimation(m))
                    .expect("failed to send from Tether Interface thread");
            }
        }
        sleep(Duration::from_millis(1));
    })
}
