use std::{net::SocketAddr, sync::mpsc};

use env_logger::Env;
use log::{debug, info};

use clap::Parser;

use crate::{
    artnet::{ArtNetInterface, ArtNetMode},
    model::Model,
    settings::Cli,
    tether_interface::start_tether_thread,
    ui::SIMPLE_WIN_SIZE,
};

mod animation;
mod artnet;
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

    let mut handles = Vec::new();

    let (tether_tx, tether_rx) = mpsc::channel();
    let tether_handle = start_tether_thread(tether_tx);

    handles.push(tether_handle);

    let artnet = {
        if cli.artnet_broadcast {
            ArtNetInterface::new(ArtNetMode::Broadcast)
        } else {
            ArtNetInterface::new(ArtNetMode::Unicast(
                SocketAddr::from((cli.unicast_src, 6453)),
                SocketAddr::from((cli.unicast_dst, 6454)),
            ))
        }
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
            initial_window_size: Some(SIMPLE_WIN_SIZE),
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
