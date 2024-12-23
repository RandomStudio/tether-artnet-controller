use std::{
    sync::{
        self,
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::anyhow;
use egui::Color32;
use log::{debug, error, info};
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
pub struct TetherKnobPayload {
    pub index: u8,
    pub position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RemoteMacroValue {
    ControlValue(f64),
    ColourValue(Color32),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMacroMessage {
    /// If no fixtures specified, assume all
    pub fixture_labels: Option<Vec<String>>,
    pub macro_label: String,
    /// Start value will be "whatever the current value is";
    /// so `target_value` is the End value
    pub value: RemoteMacroValue,
    /// Animation duration in ms
    pub ms: Option<u64>,
}
#[allow(dead_code)]
pub enum TetherMidiMessage {
    /// Already-encoded payload
    // Raw(Vec<u8>),
    NoteOn(TetherNotePayload),
    // NoteOff(TetherNotePayload),
    ControlChange(TetherControlChangePayload),
    Knob(TetherKnobPayload),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSceneMessage {
    pub scene_label: String,
    pub ms: Option<u64>,
    /// If no fixtures specified, assume all
    pub fixture_labels: Option<Vec<String>>,
}

pub enum RemoteControlMessage {
    Midi(TetherMidiMessage),
    MacroAnimation(RemoteMacroMessage),
    SceneAnimation(RemoteSceneMessage),
}

pub struct TetherInterface {
    pub message_rx: Receiver<RemoteControlMessage>,
    message_tx: Sender<RemoteControlMessage>,
}

impl TetherInterface {
    pub fn new() -> Self {
        let (message_tx, message_rx) = sync::mpsc::channel();
        // let (quit_tx, quit_rx) = sync::mpsc::channel();

        TetherInterface {
            message_tx,
            message_rx,
        }
    }

    pub fn connect(
        &mut self,
        should_quit: Arc<Mutex<bool>>,
        tether_host: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        info!("Attempt to connect Tether Agent...");

        if let Ok(tether_agent) = TetherAgentOptionsBuilder::new("ArtnetController")
            .host(tether_host)
            .build()
        {
            let input_midi_cc = PlugOptionsBuilder::create_input("controlChange")
                .build(&tether_agent)
                .expect("failed to create Input Plug");

            let input_midi_notes = PlugOptionsBuilder::create_input("notesOn")
                .build(&tether_agent)
                .expect("failed to create Input Plug");

            let input_midi_kobs = PlugOptionsBuilder::create_input("knobs")
                .build(&tether_agent)
                .expect("failed to create Input Plug");

            let input_macros = PlugOptionsBuilder::create_input("macros")
                .build(&tether_agent)
                .expect("failed to create Input Plug");

            let input_scenes = PlugOptionsBuilder::create_input("scenes")
                .build(&tether_agent)
                .expect("failed to create Input Plug");

            let tx = self.message_tx.clone();

            spawn(move || {
                while !*should_quit.lock().unwrap() {
                    while let Some((topic, message)) = tether_agent.check_messages() {
                        if input_midi_cc.matches(&topic) {
                            // debug!("MIDI CC");
                            // let m = rmp_serde::from_slice::<TetherControlChangePayload>(
                            //     message.payload(),
                            // )
                            // .unwrap();
                            // tx.send(RemoteControlMessage::Midi(
                            //     TetherMidiMessage::ControlChange(m),
                            // ))
                            // .expect("failed to send from Tether Interface thread")
                        }
                        if input_midi_notes.matches(&topic) {
                            debug!("MIDI Note");
                            let m = rmp_serde::from_slice::<TetherNotePayload>(message.payload())
                                .unwrap();
                            tx.send(RemoteControlMessage::Midi(TetherMidiMessage::NoteOn(m)))
                                .expect("failed to send from Tether Interface thread")
                        }
                        if input_macros.matches(&topic) {
                            debug!("Macro (direct) control message");
                            let m = rmp_serde::from_slice::<RemoteMacroMessage>(message.payload())
                                .unwrap();
                            tx.send(RemoteControlMessage::MacroAnimation(m))
                                .expect("failed to send from Tether Interface thread");
                        }
                        if input_scenes.matches(&topic) {
                            debug!("Remote Scene message");
                            let m = rmp_serde::from_slice::<RemoteSceneMessage>(message.payload())
                                .unwrap();
                            tx.send(RemoteControlMessage::SceneAnimation(m))
                                .expect("failed to send from Tether Interface thread");
                        }
                        if input_midi_kobs.matches(&topic) {
                            debug!("Remote Knobs (MIDI) message");
                            let m = rmp_serde::from_slice::<TetherKnobPayload>(message.payload())
                                .unwrap();
                            tx.send(RemoteControlMessage::Midi(TetherMidiMessage::Knob(m)))
                                .expect("failed to send from Tether Interface thread");
                        }
                    }
                    sleep(Duration::from_millis(1));
                }
                info!("Tether Interface: Thread loop end");
            });

            Ok(())
        } else {
            error!("Failed to connect Tether");
            Err(anyhow!("Tether failed to connect"))
        }
    }
}
