use std::{sync::mpsc, time::Duration};

use env_logger::Env;
use log::{debug, info};

use clap::Parser;

use crate::{model::Model, settings::Cli, ui::NARROW_WINDOW};

mod animation;
mod artnet;
mod model;
pub mod project;
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

    let mut model = Model::new(cli.clone());

    if cli.artnet_broadcast && (cli.unicast_src.is_some() || cli.unicast_dst.is_some()) {
        panic!("You cannot enabled Broadcast mode AND set Unicast details at the same time");
    }

    if cli.headless_mode {
        info!("Running in headless mode; Ctrl+C to quit");
        let mut should_quit = false;
        let (quit_cli_tx, quit_cli_rx) = mpsc::channel();

        ctrlc::set_handler(move || {
            quit_cli_tx
                .send(())
                .expect("failed to send quit message via channel");
        })
        .expect("failed to set Ctrl+C handler");
        std::thread::sleep(Duration::from_secs(2));
        while !should_quit {
            if quit_cli_rx.try_recv().is_ok() {
                info!("Headless loop should quit");
                should_quit = true;
            }
            std::thread::sleep(Duration::from_millis(1));
            model.update();
        }
    } else {
        info!("Running graphics mode; close the window to quit");
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(NARROW_WINDOW),
            run_and_return: true,
            ..Default::default()
        };
        eframe::run_native(
            "Tether ArtNet Controller",
            options,
            Box::new(|_cc| Box::<Model>::new(model)),
        )
        .expect("Failed to launch GUI");
        info!("GUI ended; exit soon...");
    }

    std::thread::sleep(Duration::from_secs(1));
    info!("...Exit now");
    std::process::exit(0);
}
