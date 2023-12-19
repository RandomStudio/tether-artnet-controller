use egui::{Slider, Ui};

use crate::{model::Model, settings::CHANNELS_PER_UNIVERSE};

pub fn render(model: &mut Model, ui: &mut Ui) {
    ui.heading("Tether ArtNet");
    for i in 0..CHANNELS_PER_UNIVERSE {
        ui.add(Slider::new(&mut model.channels_state[i as usize], 0..=255));
    }
}
