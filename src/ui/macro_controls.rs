use egui::{Color32, Grid, RichText, ScrollArea, Slider, Ui};

use crate::{
    artnet::{random, zero},
    model::Model,
    project::fixture::FixtureMacro,
};

pub fn render_macro_controls(model: &mut Model, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("All");
        if ui.button("HOME").clicked() {
            model.apply_macros = false;
            model.apply_home_values();
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

                    let mut any_changed = false;

                    Grid::new(format!("macros_{}", i))
                        .num_columns(3)
                        .show(ui, |ui| {
                            for m in current_mode.macros.iter_mut() {
                                match m {
                                    FixtureMacro::Control(control_macro) => {
                                        let remapped_channels: Vec<u16> = control_macro
                                            .channels
                                            .iter()
                                            .map(|c| c + fixture.offset_channels)
                                            .collect();
                                        let channel_list = format!(
                                            "{:?} => {:?}",
                                            &control_macro.channels, remapped_channels
                                        );
                                        ui.label(&control_macro.label).on_hover_text(channel_list);
                                        if ui
                                            .add_enabled(
                                                control_macro.animation.is_none(),
                                                Slider::new(
                                                    &mut control_macro.current_value,
                                                    0..=255,
                                                )
                                                .step_by(1.0),
                                            )
                                            .changed()
                                        {
                                            model.apply_macros = true;
                                            any_changed = true;
                                        };
                                        ui.small(control_macro.global_index.to_string());

                                        if let Some(animation) = &mut control_macro.animation {
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
                                    }
                                    FixtureMacro::Colour(colour_macro) => {
                                        ui.label(&colour_macro.label);
                                        ui.add_enabled_ui(colour_macro.animation.is_none(), |ui| {
                                            if ui
                                                .color_edit_button_srgba(
                                                    &mut colour_macro.current_value,
                                                )
                                                .changed()
                                            {
                                                model.apply_macros = true;
                                                any_changed = true;
                                            }
                                        });
                                        {};
                                        if let Some((animation, _start, _end)) =
                                            &mut colour_macro.animation
                                        {
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
                                    }
                                }

                                ui.end_row();
                            }
                        });

                    if any_changed {
                        if let Some(scene) = model.project.scenes.iter_mut().find(|x| x.last_active)
                        {
                            scene.is_editing = true;
                        }
                    }
                });
            }
        });
}
