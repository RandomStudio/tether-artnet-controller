use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::Receiver,
    time::Duration,
};

use artnet_protocol::{ArtCommand, Output};
use log::{debug, error};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tether_agent::{PlugDefinition, TetherAgent};

use crate::{
    project::{FixtureConfig, Project},
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    tether_interface::{TetherControlChangePayload, TetherMidiMessage, TetherNotePayload},
    ui::{render_fixture_controls, render_macro_controls, render_sliders},
};

pub struct ArtNetInterface {
    pub socket: UdpSocket,
    pub destination: SocketAddr,
}

pub struct Model {
    pub channels_state: Vec<u8>,
    pub channels_assigned: Vec<bool>,
    pub tether_rx: Receiver<TetherMidiMessage>,
    pub settings: Cli,
    pub artnet: ArtNetInterface,
    pub project: Project,
    pub selected_macro_group_index: usize,
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::SidePanel::left("LeftPanel").show(ctx, |ui| {
            render_sliders(self, ui);
        });

        egui::SidePanel::right("RightPanel").show(ctx, |ui| {
            render_macro_controls(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            render_fixture_controls(self, ui);
        });

        self.update();
    }
}

impl Model {
    pub fn new(
        tether_rx: Receiver<TetherMidiMessage>,
        settings: Cli,
        artnet: ArtNetInterface,
    ) -> Model {
        let project = Project::load("./project.json").expect("failed to load project");

        let fixtures_clone = project.clone().fixtures;

        let mut channels_assigned: Vec<bool> = [false].repeat(CHANNELS_PER_UNIVERSE as usize);
        for fc in fixtures_clone.iter() {
            if let Some(fixture) = &fc.fixture {
                let current_mode = &fixture.modes[fc.mode];
                for m in &current_mode.mappings {
                    let channel_index = m.channel + fc.offset_channels - 1;
                    channels_assigned[channel_index as usize] = true;
                }
            }
        }

        let mut model = Model {
            tether_rx,
            channels_state: Vec::new(),
            channels_assigned,
            settings,
            artnet,
            project,
            selected_macro_group_index: 0,
        };

        model.apply_channel_defaults();

        model
    }

    pub fn update(&mut self) {
        let mut work_done = false;
        let mut rng = rand::thread_rng();

        for _i in 0..CHANNELS_PER_UNIVERSE {
            if self.settings.auto_random {
                for c in self.channels_state.iter_mut() {
                    *c = rng.gen::<u8>();
                }
            }
            if self.settings.auto_zero {
                self.channels_state = [0].repeat(CHANNELS_PER_UNIVERSE as usize);
            }
        }

        while let Ok(m) = self.tether_rx.try_recv() {
            work_done = true;
            match m {
                TetherMidiMessage::Raw(_) => todo!(),
                TetherMidiMessage::NoteOn(note) => {
                    let TetherNotePayload {
                        note,
                        channel: _,
                        velocity: _,
                    } = note;
                    let start_note = 48;
                    let index = note - start_note;
                    debug!("Note {} => macro group index {}", note, index);
                    self.selected_macro_group_index = index as usize;
                }
                TetherMidiMessage::NoteOff(_) => todo!(),
                TetherMidiMessage::ControlChange(cc) => {
                    let TetherControlChangePayload {
                        channel: _,
                        controller,
                        value,
                    } = cc;

                    let active_macros = self
                        .project
                        .fixtures
                        .iter()
                        .map(|fc| {
                            if let Some(fixture) = &fc.fixture {
                                let macros = fixture.modes[0].macros.clone();
                                return Some((fc.clone(), macros));
                            } else {
                                return None;
                            }
                        })
                        .filter_map(|x| x);

                    let controller_start = 48;

                    for (i, (fixture_config, m)) in active_macros.enumerate() {
                        if self.selected_macro_group_index as usize == i {
                            debug!("Adjust for macros {:?}", m);
                            let target_macro_index = controller - controller_start;
                            debug!("Controller {} => {}", controller, target_macro_index);
                            match m.get(target_macro_index as usize) {
                                Some(macro_control) => {
                                    let value = value * 2;
                                    debug!("Adjust {:?} to {}", macro_control, value);
                                    // macro_control.current_value = value * 2;
                                    for c in &macro_control.channels {
                                        let channel_index =
                                            (*c - 1 + fixture_config.offset_channels) as usize;
                                        debug!("Set channel {} to value {}", channel_index, value);
                                        self.channels_state[channel_index] = value;
                                    }
                                }
                                None => {
                                    error!("Failed to match macro control");
                                }
                            }
                        }
                    }
                }
            }
        }

        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: self.channels_state.clone().into(),
            ..Output::default()
        });

        let buff = command.write_to_buffer().unwrap();
        self.artnet
            .socket
            .send_to(&buff, self.artnet.destination)
            .unwrap();

        if self.settings.auto_random || self.settings.auto_zero {
            std::thread::sleep(Duration::from_secs(1));
        } else {
            if !work_done {
                std::thread::sleep(Duration::from_millis(self.settings.artnet_update_frequency));
            }
        }
    }

    pub fn apply_channel_defaults(&mut self) {
        self.channels_state = [0].repeat(CHANNELS_PER_UNIVERSE as usize); // init zeroes

        let fixtures_clone = self.project.fixtures.clone();
        for fc in fixtures_clone.iter() {
            if let Some(fixture) = &fc.fixture {
                let current_mode = &fixture.modes[fc.mode];
                for m in &current_mode.mappings {
                    if let Some(default_value) = m.default {
                        let channel_index = m.channel + fc.offset_channels - 1;
                        self.channels_state[channel_index as usize] = default_value;
                    }
                }
            }
        }
    }
}
