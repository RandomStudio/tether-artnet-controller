use egui::{ScrollArea, Slider, Ui};

use crate::{model::Model, settings::CHANNELS_PER_UNIVERSE};

pub fn render_sliders(model: &mut Model, ui: &mut Ui) {
    ui.heading("Basic Slider Controls");
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for i in 0..CHANNELS_PER_UNIVERSE {
                ui.horizontal(|ui| {
                    ui.label(format!("Channel #{}", i + 1));
                    ui.add(Slider::new(&mut model.channels_state[i as usize], 0..=255));
                });
            }
        });
}

pub fn render_fixture_controls(model: &mut Model, ui: &mut Ui) {
    ui.heading("Fixtures");
    for f in model.project.fixtures.iter() {
        if let Some(fixture) = &f.fixture {
            ui.group(|ui| {
                ui.heading(&fixture.name);
                let current_mode = &fixture.modes[f.mode];
                for m in &current_mode.mappings {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}: {}", m.channel, m.label));
                        ui.add(Slider::new(
                            &mut model.channels_state[m.channel as usize],
                            0..=255,
                        ));
                    });
                }
            });
        }
    }
}
