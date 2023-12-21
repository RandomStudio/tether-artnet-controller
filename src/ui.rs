use egui::{Color32, Grid, RichText, ScrollArea, Slider, Ui};

use crate::{
    artnet::{random, zero},
    model::Model,
    settings::CHANNELS_PER_UNIVERSE,
};

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
    if ui.button("DEFAULTS").clicked() {
        model.apply_channel_defaults();
    }
    if ui.button("ZERO").clicked() {
        zero(&mut model.channels_state);
    }
    if ui.button("RANDOM").clicked() {
        random(&mut model.channels_state);
    }

    ui.separator();

    ui.heading("Macros");

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
                    ui.heading(RichText::new(&fixture.label).color(
                        if i == model.selected_macro_group_index {
                            Color32::GREEN
                        } else {
                            Color32::GRAY
                        },
                    ));
                    ui.label(&fixture.config.name);
                    let current_mode = &mut fixture.config.active_mode;

                    Grid::new(format!("macros_{}", i))
                        .num_columns(2)
                        .show(ui, |ui| {
                            for (_i, m) in current_mode.macros.iter_mut().enumerate() {
                                ui.label(&m.label);
                                if ui.add(Slider::new(&mut m.current_value, 0..=255)).changed() {
                                    model.apply_macros = true;
                                };

                                ui.end_row();
                            }
                        });
                });
            }
        });
}
