use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use eframe::App;
use env_logger::Env;
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::Parser;

use crate::{
    model::{ArtNetInterface, Model},
    settings::Cli,
};

mod model;
mod settings;
mod ui;

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("paho_mqtt", log::LevelFilter::Warn)
        .init();

    debug!("Started with settings: {:?}", cli);

    let tether_agent = TetherAgentOptionsBuilder::new("ArtnetController")
        .build()
        .expect("failed to init Tether Agent");

    let input_midi_cc = PlugOptionsBuilder::create_input("controlChange")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let src = SocketAddr::from((cli.unicast_src, 6453));
    let dst = SocketAddr::from((cli.unicast_dst, 6454));

    let socket = UdpSocket::bind(src).unwrap();

    let mut model = Model {
        tether_agent,
        channels_state: [0; 256],
        input_midi_cc,
        settings: cli.clone(),
        artnet: ArtNetInterface {
            socket,
            destination: dst,
        },
    };

    if cli.headless_mode {
        info!("Running in headless mode; Ctrl+C to quit");
        loop {
            model.update();
        }
    } else {
        info!("Running graphics mode; close the window to quit");
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1280.0, 550.)),
            ..Default::default()
        };
        eframe::run_native(
            "Tether ArtNet Controller",
            options,
            Box::new(|_cc| Box::<Model>::new(model)),
        )
        .expect("Failed to launch GUI");
        info!("GUI ended; exit now...");
        std::process::exit(0);
    }
}
