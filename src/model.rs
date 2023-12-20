use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use artnet_protocol::{ArtCommand, Output};
use log::debug;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tether_agent::{PlugDefinition, TetherAgent};

use crate::{
    project::{ControlMacro, Project},
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    ui::{render_fixture_controls, render_macro_controls, render_sliders},
};

pub struct ArtNetInterface {
    pub socket: UdpSocket,
    pub destination: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TetherControlChangePayload {
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

pub struct MacroState {
    pub control_macro: ControlMacro,
    pub current_value: u8,
}

pub struct Model {
    pub channels_state: Vec<u8>,
    pub channels_assigned: Vec<bool>,
    pub tether_agent: TetherAgent,
    pub input_midi_cc: PlugDefinition,
    pub settings: Cli,
    pub artnet: ArtNetInterface,
    pub project: Project,
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::SidePanel::left("LeftPanel").show(ctx, |ui| {
            render_sliders(self, ui);
        });

        egui::SidePanel::right("RightPanel").show(ctx, |ui| {
            render_macro_controls(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            render_fixture_controls(self, ui);
        });

        self.update();
    }
}

impl Model {
    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();

        while let Some((topic, message)) = self.tether_agent.check_messages() {
            debug!("Received message on {:?}", &topic);
            if self.input_midi_cc.matches(&topic) {
                let m = rmp_serde::from_slice::<TetherControlChangePayload>(&message.payload())
                    .unwrap();
                let TetherControlChangePayload {
                    channel,
                    controller,
                    value,
                } = m;
                self.channels_state[controller as usize] = value * 2; // MIDI channels go [0..=127]
            }
        }

        for _i in 0..CHANNELS_PER_UNIVERSE {
            if self.settings.auto_random {
                for c in self.channels_state.iter_mut() {
                    *c = rng.gen::<u8>();
                }
            }
            if self.settings.auto_zero {
                self.channels_state = [0].repeat(CHANNELS_PER_UNIVERSE as usize);
            }
        }

        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: self.channels_state.clone().into(),
            ..Output::default()
        });

        let buff = command.write_to_buffer().unwrap();
        self.artnet
            .socket
            .send_to(&buff, self.artnet.destination)
            .unwrap();

        if self.settings.auto_random || self.settings.auto_zero {
            std::thread::sleep(Duration::from_secs(1));
        } else {
            std::thread::sleep(Duration::from_millis(self.settings.artnet_update_frequency));
        }
    }
}
