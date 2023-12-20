use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc,
};

use env_logger::Env;
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::Parser;

use crate::{
    model::{ArtNetInterface, Model},
    project::Project,
    settings::{Cli, CHANNELS_PER_UNIVERSE},
    tether_interface::start_tether_thread,
};

mod model;
mod project;
mod settings;
mod tether_interface;
mod ui;

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("paho_mqtt", log::LevelFilter::Warn)
        .filter_module("egui_glow", log::LevelFilter::Warn)
        .filter_module("egui_winit", log::LevelFilter::Warn)
        .filter_module("eframe", log::LevelFilter::Warn)
        .init();

    debug!("Started with settings: {:?}", cli);

    let src = SocketAddr::from((cli.unicast_src, 6453));
    let dst = SocketAddr::from((cli.unicast_dst, 6454));

    let socket = UdpSocket::bind(src).unwrap();

    let mut handles = Vec::new();

    let (tether_tx, tether_rx) = mpsc::channel();
    let tether_handle = start_tether_thread(tether_tx);

    handles.push(tether_handle);

    let artnet = ArtNetInterface {
        socket,
        destination: dst,
    };

    let mut model = Model::new(tether_rx, cli.clone(), artnet);

    if cli.headless_mode {
        info!("Running in headless mode; Ctrl+C to quit");
        loop {
            model.update();
        }
    } else {
        info!("Running graphics mode; close the window to quit");
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1280.0, 900.)),
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
