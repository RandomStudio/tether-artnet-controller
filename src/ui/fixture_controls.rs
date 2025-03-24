use egui::{DragValue, Grid, ScrollArea, Slider, Ui};
use log::debug;

use crate::model::Model;

pub fn render_fixture_controls(model: &mut Model, ui: &mut Ui) {
    ui.heading("Fixtures");
    ui.separator();
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if model.adding_new_fixture {
                ui.group(|ui| {
                    if let Some(new_fixture) = &mut model.new_fixture_to_add {
                        // -------- Edit some options and add to project (or cancel)
                        ui.text_edit_singleline(&mut new_fixture.label);
                        ui.horizontal(|ui| {
                            ui.label("Offset channels:");
                            ui.add(
                                DragValue::new(&mut new_fixture.offset_channels)
                                    .clamp_range(0..=512)
                                    .speed(1),
                            );
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Add to Project ✅").clicked() {
                                debug!("Adding new fixture to project");
                                model.project.fixtures.insert(0, new_fixture.clone());
                                model.adding_new_fixture = false;
                            }
                            if ui.button("Cancel 🗙").clicked() {
                                model.adding_new_fixture = false;
                            }
                        });

                        if !model.adding_new_fixture {
                            // If we're no longer adding a fixture, clear the current one
                            model.new_fixture_to_add = None;
                        }
                    } else {
                        // -------- Provide a list of fixtures
                        for fixture in model.known_fixtures.iter() {
                            ui.horizontal(|ui| {
                                ui.label(&fixture.name);
                                if ui.button("Select").clicked() {
                                    model.new_fixture_to_add = Some(fixture.into());
                                }
                            });
                        }
                        if ui.button("Cancel 🗙").clicked() {
                            model.adding_new_fixture = false;
                            model.new_fixture_to_add = None;
                        }
                    }
                });
            } else if ui.button("Add Fixture ➕").clicked() {
                debug!("Add fixture");
                model.adding_new_fixture = true;
            }
            if !model.project.fixtures.is_empty() {
                ui.separator();
                fixture_controls_in_project(model, ui);
            }
        });
}

fn fixture_controls_in_project(model: &mut Model, ui: &mut Ui) {
    let mut remove_index = None;

    for (i, fixture) in model.project.fixtures.iter_mut().enumerate() {
        let config = &fixture.config;
        // ----------------
        ui.horizontal(|ui| {
            ui.heading(&fixture.label);
            if ui.button("🗑").clicked() {
                remove_index = Some(i);
            }
        });
        // ----------------
        ui.horizontal(|ui| {
            ui.label((config.name).to_string());
            ui.hyperlink_to("Reference/manual", &config.reference);
        });
        // ----------------
        ui.horizontal(|ui| {
            ui.label("Offset channels:");
            ui.add(
                DragValue::new(&mut fixture.offset_channels)
                    .clamp_range(0..=512)
                    .speed(1),
            );
        });

        // ----------------
        let current_mode = &config.modes[fixture.mode_index];
        ui.heading("Mappings");

        Grid::new(format!("mappings_{}", i))
            .num_columns(3)
            .show(ui, |ui| {
                for m in &current_mode.mappings {
                    let channel_index = m.channel + fixture.offset_channels - 1;
                    ui.horizontal(|ui| {
                        ui.label(&m.label);
                        if let Some(notes) = &m.notes {
                            ui.label("ℹ").on_hover_text(format!(
                                "#Channel {}: {}",
                                channel_index + 1,
                                notes
                            ));
                        }
                    });
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
                                if let Some(notes) = &r.notes {
                                    ui.label("ℹ").on_hover_text(notes);
                                }
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
    if let Some(index) = remove_index {
        debug!("Delete fixture with index {}", index);
        model.project.fixtures.remove(index);
    }
}
