use egui::{Color32, Grid, RichText, ScrollArea, Slider, Ui, Vec2};
use log::{error, info, warn};

use crate::{model::Model, project::Project, settings::CHANNELS_PER_UNIVERSE};

use self::{
    fixture_controls::render_fixture_controls, macro_controls::render_macro_controls,
    scenes::render_scenes,
};

mod fixture_controls;
mod macro_controls;
mod scenes;

pub const SIMPLE_WIN_SIZE: Vec2 = Vec2::new(400., 1024.0);
pub const ADVANCED_WIN_SIZE: Vec2 = Vec2::new(1280., 900.);

// const WINDOW_RESET_POSITION: [f32; 2] = [32.0, 32.0];

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
                    // frame.set_window_pos(WINDOW_RESET_POSITION.into())
                }
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Scenes, "Scenes")
                    .clicked()
                {
                    frame.set_window_size(ADVANCED_WIN_SIZE);
                    // frame.set_window_pos(WINDOW_RESET_POSITION.into())
                }
                ui.label("|");
                if ui.button("New").clicked() {
                    // TODO: ask for confirmation first!
                    warn!("Clearing current project from memory");
                    model.project = Project::new();
                    model.current_project_path = None;
                }
                match &model.current_project_path {
                    Some(existing_project_path) => {
                        if ui.button("Save").clicked() {
                            match Project::save(&existing_project_path, &model.project) {
                                Ok(()) => {
                                    info!("Saved OK!");
                                }
                                Err(e) => {
                                    error!("Error saving project: {:?}", e);
                                }
                            }
                        }
                    }
                    None => {
                        if ui.button("Save As...").clicked() {
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
                    }
                }
                if ui.button("Load").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("text", &["json"])
                        .pick_file()
                    {
                        match Project::load(&path.display().to_string()) {
                            Ok(p) => {
                                model.project = p;
                                model.current_project_path = Some(path.display().to_string());
                            }
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
