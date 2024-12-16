use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use egui::Color32;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use tween::SineInOut;

use crate::{
    animation::{animate_colour, Animation},
    artnet::{random, zero, ArtNetInterface},
    project::{
        artnetconfig::{get_artnet_interface, ArtNetConfigMode},
        fixture::{FixtureConfig, FixtureInstance, FixtureMacro},
        load_all_fixture_configs,
        scene::SceneValue,
        Project,
    },
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    tether_interface::{
        RemoteControlMessage, RemoteMacroMessage, RemoteMacroValue, RemoteSceneMessage,
        TetherControlChangePayload, TetherInterface, TetherKnobPayload, TetherMidiMessage,
        TetherNotePayload,
    },
    ui::{render_gui, ViewMode},
};

#[derive(PartialEq, Deserialize, Serialize)]
pub enum BehaviourOnExit {
    DoNothing,
    Home,
    Zero,
}

pub enum TetherStatus {
    NotConnected,
    Connected,
    Errored(String),
}

pub struct Model {
    pub settings: Cli,
    pub channels_state: Vec<u8>,
    pub channels_assigned: Vec<bool>,
    pub tether_interface: TetherInterface,
    pub tether_status: TetherStatus,
    /// A working, connected ArtNet interface, or None if disconnected
    /// and/or currently editing settings
    pub artnet: Option<ArtNetInterface>,
    /// UI for ArtNet settings; not necessarily the same
    /// as the ones in use, until actually applied
    pub artnet_edit_mode: ArtNetConfigMode,
    pub project: Project,
    /// If None, we are in a New/Unsaved project
    pub current_project_path: Option<String>,
    pub adding_new_fixture: bool,
    pub new_fixture_to_add: Option<FixtureInstance>,
    pub known_fixtures: Vec<FixtureConfig>,

    /// Whether macros should currently be applied via ArtNet output.
    /// It is important that this is _disabled_ when adjusting channel
    /// values directly, e.g. in Setup mode.
    pub apply_macros: bool,
    /// Determines which macros are adjusted via MIDI
    pub selected_macro_group_index: usize,
    pub view_mode: ViewMode,
    pub exit_mode: BehaviourOnExit,
    pub save_on_exit: bool,
    pub show_confirm_exit: bool,
    pub allowed_to_close: bool,
    pub should_quit: Arc<Mutex<bool>>,
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        render_gui(self, ctx, frame);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        debug!("eframe On exit");
        self.reset_before_quit();
    }
}

impl Model {
    pub fn new(cli: Cli) -> Model {
        let mut current_project_path = None;

        let project = match Project::load(&cli.project_path) {
            Ok(p) => {
                current_project_path = Some(String::from(&cli.project_path));
                p
            }
            Err(e) => {
                error!(
                    "Failed to load project from path \"{}\"; {:?}",
                    &cli.project_path, e
                );
                info!("Blank project will be loaded instead.");
                Project::new()
            }
        };

        let artnet = get_artnet_interface(&cli, &project);

        let fixtures_clone = project.clone().fixtures;

        let mut channels_assigned: Vec<bool> = [false].repeat(CHANNELS_PER_UNIVERSE as usize);
        for fixture in fixtures_clone.iter() {
            let current_mode = &fixture.config.modes[0];
            for m in &current_mode.mappings {
                let channel_index = m.channel + fixture.offset_channels - 1;
                channels_assigned[channel_index as usize] = true;
            }
        }

        let should_quit = Arc::new(Mutex::new(false));

        let tether_interface = TetherInterface::new();

        let should_auto_connect = !cli.tether_disable_autoconnect;

        let mut model = Model {
            tether_status: TetherStatus::NotConnected,
            tether_interface,
            channels_state: Vec::new(),
            channels_assigned,
            settings: cli,
            artnet: match artnet {
                Ok(artnet) => Some(artnet),
                Err(_) => None,
            },
            artnet_edit_mode: ArtNetConfigMode::Broadcast,
            project,
            // ----
            known_fixtures: load_all_fixture_configs(),
            adding_new_fixture: false,
            new_fixture_to_add: None,
            // ----
            current_project_path,
            selected_macro_group_index: 0,
            apply_macros: false,
            view_mode: ViewMode::Scenes,
            exit_mode: BehaviourOnExit::Home,
            save_on_exit: true,
            show_confirm_exit: false,
            allowed_to_close: false,
            should_quit,
        };

        if should_auto_connect {
            info!("Auto connect Tether enabled; will attempt to connect now...");
            attempt_connection(&mut model)
        }

        model.apply_home_values();

        model
    }

    pub fn update(&mut self) {
        let mut work_done = false;

        while let Ok(m) = self.tether_interface.message_rx.try_recv() {
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
        }
        if let Some(artnet) = &mut self.artnet {
            if artnet.update(
                &self.channels_state,
                &self.project.fixtures,
                self.apply_macros,
            ) {
                trace!("Artnet did update");
                work_done = true;
            }
        }

        if self.apply_macros {
            work_done = true;
            self.animate_macros();
            if let Some(artnet) = &self.artnet {
                self.channels_state = artnet.get_state().to_vec();
            }
        }

        if self.settings.auto_random || self.settings.auto_zero {
            std::thread::sleep(Duration::from_secs(1));
        }
        if !work_done {
            std::thread::sleep(Duration::from_millis(1));
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
                    if self.selected_macro_group_index == i {
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
            TetherMidiMessage::Knob(TetherKnobPayload { index, position }) => {
                for fixture in self.project.fixtures.iter_mut() {
                    for m in fixture.config.active_mode.macros.iter_mut() {
                        match m {
                            FixtureMacro::Control(control_macro) => {
                                if index == control_macro.global_index {
                                    control_macro.current_value = (255.0 * position) as u8;
                                }
                            }
                            FixtureMacro::Colour(_colour_macro) => {
                                // Ignore colour macros for now
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
                scene.last_active = true;
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
        debug!("Apply home values");
        debug!("Before: {:?}", self.channels_state);

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
        debug!("After: {:?}", self.channels_state);
    }

    pub fn reset_before_quit(&mut self) {
        *self.should_quit.lock().unwrap() = true;
        if self.save_on_exit {
            info!("Save-on-exit enabled; will save current project if loaded...");
            if let Some(existing_project_path) = &self.current_project_path {
                match Project::save(existing_project_path, &self.project) {
                    Ok(_) => info!("...Saved current project \"{}\" OK", &existing_project_path),
                    Err(e) => error!("...Something went wrong saving: {}", e),
                }
            } else {
                warn!("...No project was loaded; nothing saved")
            }
        }
        match self.exit_mode {
            BehaviourOnExit::DoNothing => {
                warn!("Exit behaviour explicitly set to do Nothing; will just quit")
            }
            BehaviourOnExit::Home => {
                info!("Exit Behaviour: All fixtures Go Home");
                self.apply_macros = false;
                self.apply_home_values();
                self.update();
            }
            BehaviourOnExit::Zero => {
                info!("Exit Behaviour: All fixtures Go Zero");
                self.apply_macros = false;
                zero(&mut self.channels_state);
                self.update();
            }
        }
        std::thread::sleep(Duration::from_millis(500));
        info!("...reset before quit done");
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

pub fn attempt_connection(model: &mut Model) {
    match model.tether_interface.connect(
        model.should_quit.clone(),
        model.settings.tether_host.as_deref(),
    ) {
        Ok(_) => {
            model.tether_status = TetherStatus::Connected;
        }
        Err(e) => {
            model.tether_status = TetherStatus::Errored(format!("Error: {e}"));
        }
    }
}
