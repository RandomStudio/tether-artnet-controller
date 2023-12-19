use egui::{ScrollArea, Slider, Ui};

use crate::{model::Model, settings::CHANNELS_PER_UNIVERSE};

pub fn render(model: &mut Model, ui: &mut Ui) {
    ui.heading("Tether ArtNet");
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
