use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
    thread::sleep,
    time::Duration,
};

use artnet_protocol::{ArtCommand, Output};
use env_logger::Env;
use log::{debug, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::Parser;

use crate::settings::Cli;

mod settings;
pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TetherControlChangePayload {
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

const CHANNELS_PER_UNIVERSE: u16 = 512;

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("paho_mqtt", log::LevelFilter::Warn)
        .init();

    info!("Started");
    debug!("Settings: {:?}", cli);
    let tether_agent = TetherAgentOptionsBuilder::new("ArtnetController")
        .build()
        .expect("failed to init Tether Agent");

    let src = SocketAddr::from((cli.unicast_src, 6453));
    let dst = SocketAddr::from((cli.unicast_dst, 6454));

    let socket = UdpSocket::bind(src).unwrap();

    let artnet = ArtNetInterface {
        socket,
        destination: dst,
    };

    let input_midi_cc = PlugOptionsBuilder::create_input("controlChange")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let mut rng = rand::thread_rng();

    let mut channels: Vec<u8> = Vec::with_capacity(CHANNELS_PER_UNIVERSE as usize);
    channels = [0].repeat(CHANNELS_PER_UNIVERSE as usize);

    loop {
        while let Some((topic, message)) = tether_agent.check_messages() {
            debug!("Received message on {:?}", &topic);
            if input_midi_cc.matches(&topic) {
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
            if cli.auto_random {
                channels.push(rng.gen::<u8>());
            }
            if cli.auto_zero {
                channels.push(0);
            }
        }

        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: channels.clone().into(),
            ..Output::default()
        });

        let buff = command.write_to_buffer().unwrap();
        artnet.socket.send_to(&buff, artnet.destination).unwrap();

        if cli.auto_random || cli.auto_zero {
            sleep(Duration::from_secs(1));
        } else {
            sleep(Duration::from_millis(cli.artnet_update_frequency));
        }
    }
}
