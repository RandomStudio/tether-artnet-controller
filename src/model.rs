use std::{
    sync::mpsc::Receiver,
    time::{Duration, SystemTime},
};

use egui::Color32;
use log::{debug, error, info};
use tween::SineInOut;

use crate::{
    animation::{animate_colour, Animation},
    artnet::{random, zero, ArtNetInterface},
    project::{fixture::FixtureMacro, Project, SceneValue},
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    tether_interface::{
        RemoteControlMessage, RemoteMacroMessage, RemoteMacroValue, RemoteSceneMessage,
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
    pub current_project_path: Option<String>,
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
        let mut current_project_path = None;

        let project = match Project::load(&settings.project_path) {
            Ok(p) => {
                current_project_path = Some(String::from(&settings.project_path));
                p
            }
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
            current_project_path,
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
                RemoteControlMessage::MacroAnimation(animation_msg) => {
                    self.handle_macro_message(animation_msg);
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
                // std::thread::sleep(Duration::from_secs_f32(
                //     1.0 / self.settings.artnet_update_frequency as f32,
                // ));
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    fn animate_macros(&mut self) {
        for fixture in self.project.fixtures.iter_mut() {
            for m in fixture.config.active_mode.macros.iter_mut() {
                match m {
                    FixtureMacro::Control(control_macro) => {
                        if let Some(animation) = &mut control_macro.animation {
                            let (value, is_done) = animation.get_value_and_done();
                            let dmx_value = (value * 255.0) as u8;
                            control_macro.current_value = dmx_value;

                            // NB: Check if done AFTER applying value
                            if is_done {
                                debug!("Animation done; delete");
                                control_macro.animation = None;
                            }
                        }
                    }
                    FixtureMacro::Colour(colour_macro) => {
                        if let Some((animation, start_colour, end_colour)) =
                            &mut colour_macro.animation
                        {
                            let (progress, is_done) = animation.get_value_and_done();
                            colour_macro.current_value =
                                animate_colour(start_colour, end_colour, progress);

                            // NB: Check if done AFTER applying value
                            if is_done {
                                debug!("Animation done; delete");
                                colour_macro.animation = None;
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_midi_message(&mut self, m: TetherMidiMessage) {
        match m {
            // TetherMidiMessage::Raw(_) => todo!(),
            TetherMidiMessage::NoteOn(note) => {
                let TetherNotePayload {
                    note,
                    channel: _,
                    velocity: _,
                } = note;
                let start_note = self.project.midi_config.note_start;
                let index = note - start_note;
                debug!("Note {} => macro group index {}", note, index);
                self.selected_macro_group_index = index as usize;
            }
            // TetherMidiMessage::NoteOff(_) => todo!(),
            TetherMidiMessage::ControlChange(cc) => {
                let TetherControlChangePayload {
                    channel: _,
                    controller,
                    value,
                } = cc;

                let controller_start = self.project.midi_config.controller_start;

                if controller < controller_start {
                    return;
                }

                for (i, fixture) in self.project.fixtures.iter_mut().enumerate() {
                    if self.selected_macro_group_index as usize == i {
                        let target_macro_index = controller - controller_start;
                        debug!(
                            "Controller number {} => target macro index {}",
                            controller, target_macro_index
                        );
                        match fixture
                            .config
                            .active_mode
                            .macros
                            .get_mut(target_macro_index as usize)
                        {
                            Some(m) => match m {
                                FixtureMacro::Control(control_macro) => {
                                    let value = value * 2;
                                    debug!("Adjust {} to {}", &control_macro.label, value);
                                    control_macro.current_value = value;
                                }
                                FixtureMacro::Colour(colour_macro) => {
                                    let value = value * 2;

                                    let [r, g, b, a] = colour_macro.current_value.to_array();

                                    colour_macro.current_value =
                                        Color32::from_rgba_premultiplied(r, g, b, value);

                                    debug!("Color a {} => {}", a, colour_macro.current_value.a());
                                }
                            },
                            None => {
                                error!("Failed to match macro control");
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn handle_macro_message(&mut self, msg: RemoteMacroMessage) {
        for fixture in self.project.fixtures.iter_mut() {
            if fixtures_list_contains(&msg.fixture_labels, &fixture.label) {
                if let Some(target_macro) =
                    fixture
                        .config
                        .active_mode
                        .macros
                        .iter_mut()
                        .find(|m| match m {
                            FixtureMacro::Control(m) => {
                                m.label.eq_ignore_ascii_case(&msg.macro_label)
                            }
                            FixtureMacro::Colour(m) => {
                                m.label.eq_ignore_ascii_case(&msg.macro_label)
                            }
                        })
                {
                    match target_macro {
                        FixtureMacro::Control(control_macro) => {
                            match msg.value {
                                RemoteMacroValue::ControlValue(target_value) => {
                                    if let Some(ms) = msg.ms {
                                        let duration = Duration::from_millis(ms);
                                        let start_value =
                                            control_macro.current_value as f32 / 255.0;
                                        let end_value = target_value as f32 / 255.0;

                                        control_macro.animation = Some(Animation::new(
                                            duration,
                                            start_value,
                                            end_value,
                                            Box::new(SineInOut),
                                        ));

                                        debug!(
                                            "Added Control Value animation with duration {}ms, {} -> {}",
                                            duration.as_millis(),
                                            start_value,
                                            end_value
                                        );
                                    } else {
                                        debug!(
                                            "No animation; immediately go to Control Macro value"
                                        );
                                        control_macro.animation = None; // cancel first
                                        control_macro.current_value = target_value;
                                    }
                                }
                                RemoteMacroValue::ColourValue(_) => {
                                    error!("Remote Animation Message targets Control Macro, but provides Colour Value instead");
                                }
                            }
                        }
                        FixtureMacro::Colour(colour_macro) => match msg.value {
                            RemoteMacroValue::ControlValue(_) => {
                                error!("Remote Animation Message targets Colour Macro, but provices Control Value instead");
                            }
                            RemoteMacroValue::ColourValue(target_colour) => {
                                if let Some(ms) = msg.ms {
                                    let duration = Duration::from_millis(ms);
                                    let start_value = 0.;
                                    let end_value = 1.0;

                                    let animation = Animation::new(
                                        duration,
                                        start_value,
                                        end_value,
                                        Box::new(SineInOut),
                                    );
                                    let start_colour = colour_macro.current_value;
                                    let end_colour = target_colour;

                                    debug!(
                                        "Added Colour animation with duration {}ms, {:?} => {:?}",
                                        duration.as_millis(),
                                        start_colour,
                                        end_colour
                                    );

                                    colour_macro.animation =
                                        Some((animation, start_colour, end_colour));
                                } else {
                                    debug!("No animation; immediately go to Colour Macro value");
                                    colour_macro.current_value = target_colour;
                                }
                            }
                        },
                    }
                }
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
                self.apply_scene(index, msg.ms, msg.fixture_labels);
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
                                    FixtureMacro::Control(control_macro_in_fixture) => {
                                        if let Some(macro_in_scene) = fixture_state_in_scene
                                            .get(&control_macro_in_fixture.label)
                                        {
                                            match macro_in_scene {
                                                SceneValue::ControlValue(
                                                    control_macro_in_scene,
                                                ) => {
                                                    debug!(
                                                        "With fixture {}, Scene sets control macro {} to {}",
                                                        &fixture.label,
                                                        &control_macro_in_fixture.label, control_macro_in_scene
                                                    );
                                                    if let Some(ms) = animation_ms {
                                                        debug!("Scene includes animation; animate Control Value over {}ms", ms);
                                                        control_macro_in_fixture.animation =
                                                            Some(Animation::new(
                                                                Duration::from_millis(ms),
                                                                control_macro_in_fixture
                                                                    .current_value
                                                                    as f32
                                                                    / 255.0,
                                                                *control_macro_in_scene as f32
                                                                    / 255.0,
                                                                Box::new(SineInOut),
                                                            ))
                                                    } else {
                                                        debug!("No Animation specified; change Control Value immediately");
                                                        control_macro_in_fixture.current_value =
                                                            *control_macro_in_scene;
                                                    }
                                                }
                                                SceneValue::ColourValue(_) => {
                                                    debug!("This is Colour Macro for fixture; Control Macro from scene will not apply");
                                                }
                                            }
                                        }
                                    }
                                    FixtureMacro::Colour(colour_macro_in_fixture) => {
                                        if let Some(macro_in_scene) = fixture_state_in_scene
                                            .get(&colour_macro_in_fixture.label)
                                        {
                                            match macro_in_scene {
                                                SceneValue::ControlValue(_) => {
                                                    debug!("This is Control Macro for fixture; Colour Macro from scene will not apply");
                                                }
                                                SceneValue::ColourValue(colour_macro_in_scene) => {
                                                    debug!(
                                                        "With fixture {}, Scene sets colour macro {} to {:?}",
                                                        &fixture.label,
                                                        &colour_macro_in_fixture.label, colour_macro_in_scene
                                                    );
                                                    if let Some(ms) = animation_ms {
                                                        debug!("Scene includes animation; animate Colour over {}ms", ms);
                                                        let animation = Animation::new(
                                                            Duration::from_millis(ms),
                                                            0.0,
                                                            1.0,
                                                            Box::new(SineInOut),
                                                        );
                                                        let start_colour =
                                                            colour_macro_in_fixture.current_value;
                                                        let end_colour = *colour_macro_in_scene;
                                                        colour_macro_in_fixture.animation = Some((
                                                            animation,
                                                            start_colour,
                                                            end_colour,
                                                        ));
                                                    } else {
                                                        debug!("No Animation specified; change Colour immediately");
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

fn fixtures_list_contains(search_list: &Option<Vec<String>>, label_search_string: &str) -> bool {
    if let Some(list) = search_list {
        for label in list.iter() {
            if label.eq_ignore_ascii_case(label_search_string) {
                return true;
            }
        }
        false
    } else {
        true
    }
}
