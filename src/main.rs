use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use eframe::App;
use env_logger::Env;
use log::{debug, info};
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use clap::Parser;

use crate::{
    model::{ArtNetInterface, Model},
    project::Project,
    settings::{Cli, CHANNELS_PER_UNIVERSE},
};

mod model;
mod project;
mod settings;
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

    let tether_agent = TetherAgentOptionsBuilder::new("ArtnetController")
        .build()
        .expect("failed to init Tether Agent");

    let input_midi_cc = PlugOptionsBuilder::create_input("controlChange")
        .build(&tether_agent)
        .expect("failed to create Input Plug");

    let src = SocketAddr::from((cli.unicast_src, 6453));
    let dst = SocketAddr::from((cli.unicast_dst, 6454));

    let socket = UdpSocket::bind(src).unwrap();

    let project = Project::load("./project.json").expect("failed to load project");

    let mut channels_state = Vec::with_capacity(CHANNELS_PER_UNIVERSE as usize);
    channels_state = [0].repeat(CHANNELS_PER_UNIVERSE as usize); // init zeroes
    for fc in project.clone().fixtures.iter() {
        if let Some(fixture) = &fc.fixture {
            let current_mode = &fixture.modes[fc.mode];
            for m in &current_mode.mappings {
                if let Some(default_value) = m.default {
                    channels_state[(m.channel - 1) as usize] = default_value;
                }
            }
        }
    }
    let mut model = Model {
        tether_agent,
        channels_state,
        input_midi_cc,
        settings: cli.clone(),
        artnet: ArtNetInterface {
            socket,
            destination: dst,
        },
        project,
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
