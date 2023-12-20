use egui::{Color32, Grid, RichText, ScrollArea, Slider, Ui};
use log::{debug, warn};

use crate::{model::Model, settings::CHANNELS_PER_UNIVERSE};

pub fn render_sliders(model: &mut Model, ui: &mut Ui) {
    ui.heading("Global Slider Controls");

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Grid::new("sliders").num_columns(2).show(ui, |ui| {
                for i in 0..CHANNELS_PER_UNIVERSE {
                    // ui.horizontal(|ui| {
                    let text = format!("Channel #{}", i + 1);
                    let is_assigned = model.channels_assigned[i as usize];
                    ui.label(RichText::new(text).color(if is_assigned {
                        Color32::GREEN
                    } else {
                        Color32::GRAY
                    }));
                    ui.add(Slider::new(&mut model.channels_state[i as usize], 0..=255));
                    // });
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
            for (i, f) in model.project.fixtures.iter().enumerate() {
                if let Some(fixture) = &f.fixture {
                    ui.group(|ui| {
                        ui.heading(format!("{} +{}", &fixture.name, f.offset_channels));
                        ui.hyperlink_to("Reference/manual", &fixture.reference);
                        let current_mode = &fixture.modes[f.mode];

                        ui.heading("Mappings");

                        Grid::new(format!("mappings_{}", i))
                            .num_columns(2)
                            .show(ui, |ui| {
                                for m in &current_mode.mappings {
                                    let channel_index = m.channel + f.offset_channels - 1;
                                    ui.label(&m.label)
                                        .on_hover_text(format!("#Channel {}", channel_index + 1));
                                    ui.add(Slider::new(
                                        &mut model.channels_state[(channel_index) as usize],
                                        0..=255,
                                    ));
                                    ui.end_row();
                                }
                            });
                    });
                }
            }
        });
}

pub fn render_macro_controls(model: &mut Model, ui: &mut Ui) {
    ui.heading("Macros");

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, f) in model.project.fixtures.iter_mut().enumerate() {
                if let Some(fixture) = &mut f.fixture {
                    ui.group(|ui| {
                        ui.heading(&f.label);
                        ui.label(&fixture.name);
                        let current_mode = &mut fixture.modes[f.mode];

                        Grid::new(format!("macros_{}", i))
                            .num_columns(2)
                            .show(ui, |ui| {
                                for m in current_mode.macros.iter_mut() {
                                    ui.label(&m.label);
                                    if ui.add(Slider::new(&mut m.current_value, 0..=255)).changed()
                                    {
                                        for c in &m.channels {
                                            model.channels_state
                                                [(*c - 1 + f.offset_channels) as usize] =
                                                m.current_value;
                                        }
                                    }

                                    ui.end_row();
                                }
                            });
                    });
                }
            }
        });
}
