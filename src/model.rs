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
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    ui::render,
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

pub struct Model {
    pub channels_state: [u8; 256],
    pub tether_agent: TetherAgent,
    pub input_midi_cc: PlugDefinition,
    pub settings: Cli,
    pub artnet: ArtNetInterface,
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            render(self, ui);
        });
    }
}

impl Model {
    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();

        let mut channels: Vec<u8> = Vec::with_capacity(CHANNELS_PER_UNIVERSE as usize);
        channels = [0].repeat(CHANNELS_PER_UNIVERSE as usize);

        loop {
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
                    channels[controller as usize] = value * 2; // MIDI channels go [0..=127]
                }
            }

            for _i in 0..CHANNELS_PER_UNIVERSE {
                if self.settings.auto_random {
                    channels.push(rng.gen::<u8>());
                }
                if self.settings.auto_zero {
                    channels.push(0);
                }
            }

            let command = ArtCommand::Output(Output {
                port_address: 0.into(),
                data: channels.clone().into(),
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
}
