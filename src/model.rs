use std::{
    ops::Deref,
    sync::mpsc::Receiver,
    time::{Duration, SystemTime},
};

use log::{debug, error, info};
use tween::SineInOut;

use crate::{
    animation::Animation,
    artnet::{random, zero, ArtNetInterface},
    project::{FixtureInstance, Project, Scene},
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    tether_interface::{
        RemoteAnimationMessage, RemoteControlMessage, RemoteMacroMessage, RemoteSceneMessage,
        TetherControlChangePayload, TetherMidiMessage, TetherNotePayload,
    },
    ui::{render_gui, ViewMode},
};

pub struct Model {
    pub channels_state: Vec<u8>,
    pub channels_assigned: Vec<bool>,
    pub tether_rx: Receiver<RemoteControlMessage>,
    pub settings: Cli,
    pub artnet: ArtNetInterface,
    pub project: Project,
    /// Whether macros should currently be applied
    pub apply_macros: bool,
    /// Determines which macros are adjusted via MIDI
    pub selected_macro_group_index: usize,
    pub view_mode: ViewMode,
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        render_gui(self, ctx, frame);
    }
}

impl Model {
    pub fn new(
        tether_rx: Receiver<RemoteControlMessage>,
        settings: Cli,
        artnet: ArtNetInterface,
    ) -> Model {
        let project = match Project::load(&settings.project_path) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    "Failed to load project from path \"{}\"; {:?}",
                    &settings.project_path, e
                );
                info!("Blank project will be loaded instead.");
                Project::new()
            }
        };

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
            view_mode: ViewMode::Simple,
        };

        model.apply_home_values();

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
                RemoteControlMessage::SceneAnimation(scene_msg) => {
                    self.handle_scene_message(scene_msg);
                }
            }
        }

        if self.settings.auto_random {
            random(&mut self.channels_state);
        } else if self.settings.auto_zero {
            zero(&mut self.channels_state);
        } else {
            if self.artnet.update(
                &self.channels_state,
                &self.project.fixtures,
                self.apply_macros,
            ) {
                work_done = true;
                if self.apply_macros {
                    self.animate_macros();
                    self.channels_state = self.artnet.get_state().to_vec();
                }
            }
        }

        if self.settings.auto_random || self.settings.auto_zero {
            std::thread::sleep(Duration::from_secs(1));
        } else {
            if !work_done {
                // std::thread::sleep(Duration::from_millis(self.settings.artnet_update_frequency));
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    fn animate_macros(&mut self) {
        for fixture in self.project.fixtures.iter_mut() {
            for m in fixture.config.active_mode.macros.iter_mut() {
                // if let Some(animation) = &mut m.animation {
                //     let (value, is_done) = animation.get_value_and_done();
                //     let dmx_value = (value * 255.0) as u8;
                //     // m.current_value = dmx_value;

                //     // Check if done AFTER applying value
                //     if is_done {
                //         // m.animation = None;
                //     }
                // }
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

    fn handle_macro_message(&mut self, msg: RemoteMacroMessage) {
        let target_fixtures = get_target_fixtures_list(&self.project.fixtures, &msg.fixture_label);

        for (i, fixture) in self.project.fixtures.iter_mut().enumerate() {
            if target_fixtures.contains(&i) {
                // if let Some(target_macro) = fixture.config.active_mode.macros.iter_mut().find(
                //     |m: &&mut crate::project::ChannelMacro| {
                //         m.label.eq_ignore_ascii_case(&msg.macro_label)
                //     },
                // ) {
                //     target_macro.current_value = msg.value;
                // }
            }
        }
    }

    pub fn handle_animation_message(&mut self, msg: RemoteAnimationMessage) {
        let target_fixtures = get_target_fixtures_list(&self.project.fixtures, &msg.fixture_label);

        debug!(
            "Applying animation message to {} fixtures...",
            target_fixtures.len()
        );

        for (i, fixture) in self.project.fixtures.iter_mut().enumerate() {
            if target_fixtures.contains(&i) {
                // if let Some(target_macro) = fixture.config.active_mode.macros.iter_mut().find(
                //     |m: &&mut crate::project::ChannelMacro| {
                //         m.label.eq_ignore_ascii_case(&msg.macro_label)
                //     },
                // ) {
                //     let start_value = target_macro.current_value as f32 / 255.0;
                //     let end_value = msg.target_value as f32 / 255.0;
                //     let duration = Duration::from_millis(msg.duration);

                //     target_macro.animation = Some(Animation::new(
                //         duration,
                //         start_value,
                //         end_value,
                //         Box::new(SineInOut),
                //     ));

                //     debug!(
                //         "Added animation with duration {}ms, {} -> {}",
                //         duration.as_millis(),
                //         start_value,
                //         end_value
                //     );
                // }
            }
        }
    }

    pub fn handle_scene_message(&mut self, msg: RemoteSceneMessage) {
        match self
            .project
            .scenes
            .iter_mut()
            .enumerate()
            .find(|(_i, s)| s.label.eq_ignore_ascii_case(&msg.scene_label))
        {
            Some((index, scene)) => {
                debug!("Found scene \"{}\" at index {}", &scene.label, index);
                scene.last_active = Some(SystemTime::now());
                self.apply_scene(index, msg.ms, msg.fixture_filters);
            }
            None => error!("Failed to find matching scene for \"{}\"", &msg.scene_label),
        }
    }

    pub fn apply_scene(
        &mut self,
        scene_index: usize,
        animation_ms: Option<u64>,
        fixture_filters: Option<Vec<String>>,
    ) {
        match self.project.scenes.get(scene_index) {
            Some(scene) => {
                debug!("Match scene {}", &scene.label);
                for fixture in self.project.fixtures.iter_mut() {
                    for (fixture_label_in_scene, fixture_state_in_scene) in scene.state.iter() {
                        // If there are fixtureFilters applied, check for matches against this list
                        // as well as the name vs the key in the Scene. If no filters, just check
                        // the name.
                        let is_target_fixture = if let Some(filters) = &fixture_filters {
                            filters.contains(fixture_label_in_scene)
                                && fixture_label_in_scene.eq_ignore_ascii_case(&fixture.label)
                        } else {
                            fixture_label_in_scene.eq_ignore_ascii_case(&fixture.label)
                        };
                        if is_target_fixture {
                            debug!(
                                "Scene has match for fixture {} == {}",
                                &fixture.label, fixture_label_in_scene
                            );
                            for m in fixture.config.active_mode.macros.iter_mut() {
                                match m {
                                    crate::project::FixtureMacro::Control(
                                        control_macro_in_fixture,
                                    ) => {
                                        if let Some(macro_in_scene) = fixture_state_in_scene
                                            .get(&control_macro_in_fixture.label)
                                        {
                                            match macro_in_scene {
                                                crate::project::SceneValue::ControlValue(
                                                    control_macro_in_scene,
                                                ) => {
                                                    debug!(
                                                        "With fixture {}, Scene sets control macro {} to {}",
                                                        &fixture.label,
                                                        &control_macro_in_fixture.label, control_macro_in_scene
                                                    );
                                                    control_macro_in_fixture.current_value =
                                                        *control_macro_in_scene;
                                                }
                                                crate::project::SceneValue::ColourValue(_) => {
                                                    debug!("This is Colour Macro for fixture; Control Macro from scene will not apply");
                                                }
                                            }
                                        }
                                    }
                                    crate::project::FixtureMacro::Colour(
                                        colour_macro_in_fixture,
                                    ) => {
                                        if let Some(macro_in_scene) = fixture_state_in_scene
                                            .get(&colour_macro_in_fixture.label)
                                        {
                                            match macro_in_scene {
                                                crate::project::SceneValue::ControlValue(_) => {
                                                    debug!("This is Control Macro for fixture; Colour Macro from scene will not apply");
                                                }
                                                crate::project::SceneValue::ColourValue(
                                                    colour_macro_in_scene,
                                                ) => {
                                                    debug!(
                                                        "With fixture {}, Scene sets colour macro {} to {:?}",
                                                        &fixture.label,
                                                        &colour_macro_in_fixture.label, colour_macro_in_scene
                                                    );
                                                    colour_macro_in_fixture.current_value =
                                                        *colour_macro_in_scene;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.apply_macros = true;
            }
            None => {
                error!("Failed to find scene with index {}", scene_index);
            }
        }
    }

    pub fn apply_home_values(&mut self) {
        self.channels_state = [0].repeat(CHANNELS_PER_UNIVERSE as usize); // init zeroes

        let fixtures_clone = self.project.fixtures.clone();
        for fixture in fixtures_clone.iter() {
            let current_mode = &fixture.config.active_mode;
            for m in &current_mode.mappings {
                if let Some(default_value) = m.home {
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
