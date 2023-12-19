use egui::{Grid, ScrollArea, Slider, Ui};

use crate::{model::Model, settings::CHANNELS_PER_UNIVERSE};

pub fn render_sliders(model: &mut Model, ui: &mut Ui) {
    ui.heading("Global Slider Controls");
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Grid::new("sliders").num_columns(2).show(ui, |ui| {
                for i in 0..CHANNELS_PER_UNIVERSE {
                    // ui.horizontal(|ui| {
                    ui.label(format!("Channel #{}", i + 1));
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
            for f in model.project.fixtures.iter() {
                if let Some(fixture) = &f.fixture {
                    ui.group(|ui| {
                        ui.heading(&fixture.name);
                        let current_mode = &fixture.modes[f.mode];

                        ui.heading("Mappings");

                        Grid::new("mappings").num_columns(2).show(ui, |ui| {
                            for m in &current_mode.mappings {
                                ui.label(&m.label)
                                    .on_hover_text(format!("#Channel {}", m.channel));
                                ui.add(Slider::new(
                                    &mut model.channels_state[(m.channel - 1) as usize],
                                    0..=255,
                                ));
                                ui.end_row();
                            }
                        });

                        ui.separator();

                        ui.heading("Groups");
                        Grid::new("mappings").num_columns(2).show(ui, |ui| {
                            for g in &current_mode.groups {
                                ui.label(&g.label);
                            }
                        });
                    });
                }
            }
        });
}
