use egui::{Color32, Grid, RichText, ScrollArea, Slider, Ui, Vec2};
use log::{error, info, warn};

use crate::{
    artnet::{random, zero},
    model::Model,
    project::Project,
    settings::CHANNELS_PER_UNIVERSE,
};

pub const SIMPLE_WIN_SIZE: Vec2 = Vec2::new(400., 1024.0);
pub const ADVANCED_WIN_SIZE: Vec2 = Vec2::new(1280., 900.);

#[derive(PartialEq)]
pub enum ViewMode {
    Simple,
    Advanced,
    Scenes,
}

pub fn render_gui(model: &mut Model, ctx: &egui::Context, frame: &mut eframe::Frame) {
    ctx.request_repaint();

    render_mode_switcher(model, ctx, frame);

    match model.view_mode {
        ViewMode::Advanced => {
            egui::SidePanel::left("LeftPanel").show(ctx, |ui| {
                render_macro_controls(model, ui);
            });

            egui::SidePanel::right("RightPanel").show(ctx, |ui| {
                render_sliders(model, ui);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                render_fixture_controls(model, ui);
            });
        }
        ViewMode::Simple => {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_macro_controls(model, ui);
            });
        }
        ViewMode::Scenes => {
            egui::SidePanel::left("LeftPanel").show(ctx, |ui| {
                render_macro_controls(model, ui);
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                render_scenes(model, ui);
            });
        }
    }

    model.update();
}

pub fn render_mode_switcher(model: &mut Model, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("Tabs")
        .min_height(32.)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🗖");
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Simple, "Simple")
                    .clicked()
                {
                    frame.set_window_size(SIMPLE_WIN_SIZE);
                };
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Advanced, "Advanced")
                    .clicked()
                {
                    frame.set_window_size(ADVANCED_WIN_SIZE);
                    frame.set_window_pos([0., 0.].into())
                }
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Scenes, "Scenes")
                    .clicked()
                {
                    frame.set_window_size(ADVANCED_WIN_SIZE);
                    frame.set_window_pos([0., 0.].into())
                }
                ui.label("|");
                if ui.button("New").clicked() {
                    // TODO: ask for confirmation first!
                    warn!("Clearing current project from memory");
                    model.project = Project::new();
                }
                if ui.button("Save").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("text", &["json"])
                        .save_file()
                    {
                        match Project::save(&path.display().to_string(), &model.project) {
                            Ok(()) => {
                                info!("Saved OK!");
                            }
                            Err(e) => {
                                error!("Error saving project: {:?}", e);
                            }
                        }
                    }
                }
                if ui.button("Load").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("text", &["json"])
                        .pick_file()
                    {
                        match Project::load(&path.display().to_string()) {
                            Ok(p) => model.project = p,
                            Err(e) => {
                                error!(
                                    "Failed to load project from path \"{}\"; {:?}",
                                    &path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            });
        });
}

pub fn render_sliders(model: &mut Model, ui: &mut Ui) {
    ui.heading("Global Slider Controls");

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Grid::new("sliders").num_columns(2).show(ui, |ui| {
                for i in 0..CHANNELS_PER_UNIVERSE {
                    let text = format!("Channel #{}", i + 1);
                    let is_assigned = model.channels_assigned[i as usize];
                    ui.label(RichText::new(text).color(if is_assigned {
                        Color32::GREEN
                    } else {
                        Color32::GRAY
                    }));
                    if ui
                        .add(Slider::new(&mut model.channels_state[i as usize], 0..=255))
                        .changed()
                    {
                        model.apply_macros = false;
                    };
                    ui.end_row();
                }
            });
        });
}

pub fn render_fixture_controls(model: &mut Model, ui: &mut Ui) {
    ui.heading("Fixtures");
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, fixture) in model.project.fixtures.iter().enumerate() {
                let config = &fixture.config;
                ui.heading(format!("{} +{}", &fixture.label, fixture.offset_channels));
                ui.label(format!("{}", &config.name));
                ui.hyperlink_to("Reference/manual", &config.reference);
                let current_mode = &config.modes[fixture.mode_index];

                ui.heading("Mappings");

                Grid::new(format!("mappings_{}", i))
                    .num_columns(3)
                    .show(ui, |ui| {
                        for m in &current_mode.mappings {
                            let channel_index = m.channel + fixture.offset_channels - 1;
                            ui.label(&m.label).on_hover_text(format!(
                                "#Channel {}: {}",
                                channel_index + 1,
                                &m.notes.as_deref().unwrap_or_default()
                            ));
                            if ui
                                .add(Slider::new(
                                    &mut model.channels_state[(channel_index) as usize],
                                    0..=255,
                                ))
                                .changed()
                            {
                                model.apply_macros = false;
                            };
                            if let Some(range_sections) = &m.ranges {
                                ui.label("Mode/Programme:");
                                let current_range = range_sections.iter().find(|x| {
                                    let [start, end] = x.range;
                                    model.channels_state[(channel_index) as usize] >= start
                                        && model.channels_state[(channel_index) as usize] <= end
                                });
                                match current_range {
                                    Some(r) => {
                                        ui.label(&r.label);
                                    }
                                    None => {
                                        ui.label("Invalid range");
                                    }
                                }
                            } else {
                                ui.label("");
                            }
                            ui.end_row();
                        }
                    });
                ui.separator();
            }
        });
}

pub fn render_macro_controls(model: &mut Model, ui: &mut Ui) {
    ui.heading("All");
    ui.horizontal(|ui| {
        if ui.button("DEFAULTS").clicked() {
            model.apply_macros = false;
            model.apply_channel_defaults();
        }
        if ui.button("ZERO").clicked() {
            model.apply_macros = false;
            zero(&mut model.channels_state);
        }
        if ui.button("RANDOM").clicked() {
            model.apply_macros = false;
            random(&mut model.channels_state);
        }
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.heading("Macros");
        ui.label(if model.apply_macros {
            RichText::new("active").color(Color32::DARK_GREEN)
        } else {
            RichText::new("inactive").color(Color32::GRAY)
        });
    });

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, fixture) in model.project.fixtures.iter_mut().enumerate() {
                ui.group(|ui| {
                    let mut this_selected = model.selected_macro_group_index == i;
                    if ui
                        .toggle_value(&mut this_selected, "MIDI Control Target")
                        .clicked()
                    {
                        model.selected_macro_group_index = i;
                    }
                    ui.heading(&fixture.label);
                    ui.label(&fixture.config.name);
                    let current_mode = &mut fixture.config.active_mode;

                    Grid::new(format!("macros_{}", i))
                        .num_columns(3)
                        .show(ui, |ui| {
                            for m in current_mode.macros.iter_mut() {
                                let remapped_channels: Vec<u16> = m
                                    .channels
                                    .iter()
                                    .map(|c| c + fixture.offset_channels)
                                    .collect();
                                let channel_list =
                                    format!("{:?} => {:?}", &m.channels, remapped_channels);
                                ui.label(&m.label).on_hover_text(channel_list);
                                if ui
                                    .add_enabled(
                                        m.animation.is_none(),
                                        Slider::new(&mut m.current_value, 0..=255),
                                    )
                                    .changed()
                                {
                                    model.apply_macros = true;
                                };

                                if let Some(animation) = &mut m.animation {
                                    ui.label(
                                        RichText::new(format!(
                                            "{}%",
                                            (animation.get_progress() * 100.) as u8
                                        ))
                                        .color(Color32::GREEN)
                                        .small(),
                                    );
                                } else {
                                    ui.label("");
                                }

                                ui.end_row();
                            }
                        });
                });
            }
        });
}

fn render_scenes(model: &mut Model, ui: &mut Ui) {
    ui.heading("Scenes");

    ui.separator();

    let mut go_scenes = Vec::new();

    for (scene_index, scene) in model.project.scenes.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.heading(&scene.label);
            ui.horizontal(|ui| {
                if ui.button("Go").clicked() {
                    go_scenes.push(scene_index)
                    // model.apply_scene(scene_index);
                };
            });

            for (fixture_index, s) in scene.state.iter_mut().enumerate() {
                let (fixture_label, states) = s;
                ui.label(fixture_label);
                Grid::new(format!("scene-{}-state-{}", scene_index, fixture_index))
                    .num_columns(2)
                    .show(ui, |ui| {
                        for m in states.iter_mut() {
                            let (macro_label, value) = m;
                            ui.label(macro_label);
                            ui.add(Slider::new(value, 0..=255));
                            ui.end_row();
                        }
                    });
                ui.separator();
            }
        });
    }

    for scene_index in go_scenes {
        model.apply_scene(scene_index);
    }
}
