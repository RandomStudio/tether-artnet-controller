use std::{sync::mpsc::Receiver, time::Duration};

use log::debug;
use tween::SineInOut;

use crate::{
    animation::Animation,
    artnet::{random, zero, ArtNetInterface},
    project::{FixtureInstance, Project},
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    tether_interface::{
        RemoteControlMessage, TetherAnimationMessage, TetherControlChangePayload,
        TetherMacroMessage, TetherMidiMessage, TetherNotePayload,
    },
    ui::{render_fixture_controls, render_macro_controls, render_sliders},
};

pub struct Model {
    pub channels_state: Vec<u8>,
    pub channels_assigned: Vec<bool>,
    pub tether_rx: Receiver<RemoteControlMessage>,
    pub settings: Cli,
    pub artnet: ArtNetInterface,
    pub project: Project,
    pub apply_macros: bool,
    /// Determines which macros are adjusted via MIDI
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
        tether_rx: Receiver<RemoteControlMessage>,
        settings: Cli,
        artnet: ArtNetInterface,
    ) -> Model {
        let project = Project::load("./project.json").expect("failed to load project");

        let fixtures_clone = project.clone().fixtures;

        let mut channels_assigned: Vec<bool> = [false].repeat(CHANNELS_PER_UNIVERSE as usize);
        for fixture in fixtures_clone.iter() {
            let current_mode = &fixture.config.modes[0];
            for m in &current_mode.mappings {
                let channel_index = m.channel + fixture.offset_channels - 1;
                channels_assigned[channel_index as usize] = true;
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
            apply_macros: false,
        };

        model.apply_channel_defaults();

        model
    }

    pub fn update(&mut self) {
        let mut work_done = false;

        while let Ok(m) = self.tether_rx.try_recv() {
            work_done = true;
            self.apply_macros = true;
            match m {
                RemoteControlMessage::Midi(midi_msg) => {
                    self.handle_midi_message(midi_msg);
                }
                RemoteControlMessage::MacroDirect(macro_msg) => {
                    self.handle_macro_message(macro_msg);
                }
                RemoteControlMessage::MacroAnimation(animation_msg) => {
                    self.handle_animation_message(animation_msg);
                }
            }
        }

        if self.settings.auto_random {
            random(&mut self.channels_state);
        } else if self.settings.auto_zero {
            zero(&mut self.channels_state);
        } else {
            self.artnet.update(
                &self.channels_state,
                &self.project.fixtures,
                self.apply_macros,
            );
            if self.apply_macros {
                self.animate_macros();
                self.channels_state = self.artnet.get_state().to_vec();
            }
        }

        if self.settings.auto_random || self.settings.auto_zero {
            std::thread::sleep(Duration::from_secs(1));
        } else {
            if !work_done {
                std::thread::sleep(Duration::from_millis(self.settings.artnet_update_frequency));
            }
        }
    }

    fn animate_macros(&mut self) {
        for fixture in self.project.fixtures.iter_mut() {
            for m in fixture.config.active_mode.macros.iter_mut() {
                if let Some(animation) = &mut m.animation {
                    let (value, is_done) = animation.get_progress_and_done();
                    let dmx_value = (value * 255.0) as u8;
                    m.current_value = dmx_value;

                    // Check if done AFTER applying value
                    if is_done {
                        m.animation = None;
                    }
                }
            }
        }
    }

    fn handle_midi_message(&mut self, m: TetherMidiMessage) {
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

                // TODO: reimplement remote via Tether-MIDI

                // let active_macros = self
                //     .project
                //     .fixtures
                //     .iter()
                //     .map(|fc| {
                //         if let Some(fixture) = &fc.fixture {
                //             let macros = fixture.modes[0].macros.clone();
                //             return Some((fc.clone(), macros));
                //         } else {
                //             return None;
                //         }
                //     })
                //     .filter_map(|x| x);

                // let controller_start = 48;

                // for (i, (fixture_config, m)) in active_macros.enumerate() {
                //     if self.selected_macro_group_index as usize == i {
                //         debug!("Adjust for macros {:?}", m);
                //         let target_macro_index = controller - controller_start;
                //         debug!("Controller {} => {}", controller, target_macro_index);
                //         match m.get(target_macro_index as usize) {
                //             Some(macro_control) => {
                //                 let value = value * 2;
                //                 debug!("Adjust {:?} to {}", macro_control, value);
                //                 // macro_control.current_value = value * 2;
                //                 for c in &macro_control.channels {
                //                     let channel_index =
                //                         (*c - 1 + fixture_config.offset_channels) as usize;
                //                     debug!("Set channel {} to value {}", channel_index, value);
                //                     self.channels_state[channel_index] = value;
                //                 }
                //             }
                //             None => {
                //                 error!("Failed to match macro control");
                //             }
                //         }
                //     }
                // }
            }
        }
    }

    fn handle_macro_message(&mut self, msg: TetherMacroMessage) {
        let target_fixtures = get_target_fixtures_list(&self.project.fixtures, &msg.fixture_label);

        for (i, fixture) in self.project.fixtures.iter_mut().enumerate() {
            if target_fixtures.contains(&i) {
                if let Some(target_macro) = fixture.config.active_mode.macros.iter_mut().find(
                    |m: &&mut crate::project::ControlMacro| {
                        m.label.eq_ignore_ascii_case(&msg.macro_label)
                    },
                ) {
                    target_macro.current_value = msg.value;
                }
            }
        }
    }

    pub fn handle_animation_message(&mut self, msg: TetherAnimationMessage) {
        let target_fixtures = get_target_fixtures_list(&self.project.fixtures, &msg.fixture_label);

        debug!(
            "Applying animation message to {} fixtures...",
            target_fixtures.len()
        );

        for (i, fixture) in self.project.fixtures.iter_mut().enumerate() {
            if target_fixtures.contains(&i) {
                if let Some(target_macro) = fixture.config.active_mode.macros.iter_mut().find(
                    |m: &&mut crate::project::ControlMacro| {
                        m.label.eq_ignore_ascii_case(&msg.macro_label)
                    },
                ) {
                    let start_value = target_macro.current_value as f32 / 255.0;
                    let end_value = msg.target_value as f32 / 255.0;
                    let duration = Duration::from_millis(msg.duration);

                    target_macro.animation = Some(Animation::new(
                        duration,
                        start_value,
                        end_value,
                        Box::new(SineInOut),
                    ));

                    debug!(
                        "Added animation with duration {}ms, {} -> {}",
                        duration.as_millis(),
                        start_value,
                        end_value
                    );
                }
            }
        }
    }

    pub fn apply_channel_defaults(&mut self) {
        self.channels_state = [0].repeat(CHANNELS_PER_UNIVERSE as usize); // init zeroes

        let fixtures_clone = self.project.fixtures.clone();
        for fixture in fixtures_clone.iter() {
            let current_mode = &fixture.config.active_mode;
            for m in &current_mode.mappings {
                if let Some(default_value) = m.default {
                    let channel_index = m.channel + fixture.offset_channels - 1;
                    self.channels_state[channel_index as usize] = default_value;
                }
            }
        }
    }
}

fn get_target_fixtures_list(
    fixtures: &[FixtureInstance],
    label_search_string: &Option<String>,
) -> Vec<usize> {
    fixtures
        .iter()
        .enumerate()
        .filter(|(i, f)| {
            if let Some(label) = label_search_string {
                f.label.eq_ignore_ascii_case(&label)
            } else {
                true // match all
            }
        })
        .filter_map(|(i, _f)| Some(i))
        .collect()
}
