use egui::{Color32, Grid, RichText, ScrollArea, Slider, Ui, Vec2};
use log::{error, info, warn};

use crate::{
    model::{BehaviourOnExit, Model, TetherStatus},
    project::Project,
    settings::CHANNELS_PER_UNIVERSE,
};

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

pub fn render_gui(model: &mut Model, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
    ctx.request_repaint();

    if ctx.input(|i| i.viewport().close_requested()) {
        if model.allowed_to_close {
            // do nothing - we will close
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            model.show_confirm_exit = true;
        }
    }

    render_mode_switcher(model, ctx, frame);

    match model.view_mode {
        ViewMode::Advanced => {
            egui::SidePanel::left("LeftPanel").show(ctx, |ui| {
                render_tether_controls(model, ui);
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
                render_tether_controls(model, ui);
                render_macro_controls(model, ui);
            });
        }
        ViewMode::Scenes => {
            egui::SidePanel::left("LeftPanel").show(ctx, |ui| {
                render_tether_controls(model, ui);
                render_macro_controls(model, ui);
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                render_scenes(model, ui);
            });
        }
    }

    if model.show_confirm_exit {
        egui::Window::new("Ready to Quit?")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Yes ⏏").heading()).clicked() {
                        model.show_confirm_exit = false;
                        model.allowed_to_close = true;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button(RichText::new("Cancel 🗙").heading()).clicked() {
                        model.show_confirm_exit = false;
                        model.allowed_to_close = false;
                    }
                });
                ui.checkbox(&mut model.save_on_exit, "Save Project on exit");
                ui.group(|ui| {
                    ui.heading("Behaviour on exit");
                    ui.radio_value(
                        &mut model.exit_mode,
                        BehaviourOnExit::DoNothing,
                        "Do nothing",
                    );
                    ui.radio_value(
                        &mut model.exit_mode,
                        BehaviourOnExit::Home,
                        "All fixtures to Home",
                    );
                    ui.radio_value(
                        &mut model.exit_mode,
                        BehaviourOnExit::Zero,
                        "All channels to Zero",
                    );
                });
            });
    } else {
        model.update();
    }
}

pub fn render_mode_switcher(
    model: &mut Model,
    ctx: &eframe::egui::Context,
    _frame: &mut eframe::Frame,
) {
    egui::TopBottomPanel::top("Tabs")
        .min_height(32.)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🗖");
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Simple, "Simple")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(SIMPLE_WIN_SIZE));
                };
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Advanced, "Advanced")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(ADVANCED_WIN_SIZE));
                }
                if ui
                    .selectable_value(&mut model.view_mode, ViewMode::Scenes, "Scenes")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(ADVANCED_WIN_SIZE));
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
                            match Project::save(existing_project_path, &model.project) {
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
            if let Some(existing_project_path) = &model.current_project_path {
                ui.label(RichText::new(existing_project_path).italics().small());
            }
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

fn render_tether_controls(model: &mut Model, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("Tether");

        match &model.tether_status {
            TetherStatus::NotConnected => {
                ui.label(RichText::new("Not (yet) connected").color(Color32::YELLOW));
                offer_tether_connect(model, ui);
            }
            TetherStatus::Connected => {
                ui.label(RichText::new("Connected").color(Color32::LIGHT_GREEN));
            }
            TetherStatus::Errored(msg) => {
                ui.label(RichText::new(msg).color(Color32::RED));
                offer_tether_connect(model, ui);
            }
        }
    });
    ui.separator();
}

fn offer_tether_connect(model: &mut Model, ui: &mut Ui) {
    if ui.button("Connect").clicked() {
        attempt_connection(model);
    }
}

pub fn attempt_connection(model: &mut Model) {
    match model.tether_interface.connect(model.should_quit.clone()) {
        Ok(_) => {
            model.tether_status = TetherStatus::Connected;
        }
        Err(e) => {
            model.tether_status = TetherStatus::Errored(format!("Error: {e}"));
        }
    }
}
