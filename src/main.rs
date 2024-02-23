use std::{net::SocketAddr, sync::mpsc, time::Duration};

use env_logger::Env;
use log::{debug, error, info};

use clap::Parser;

use crate::{
    artnet::{ArtNetInterface, ArtNetMode},
    model::Model,
    settings::Cli,
    tether_interface::start_tether_thread,
    ui::SIMPLE_WIN_SIZE,
};

use ctrlc;

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

    let mut handles = Vec::new();

    let (tether_tx, tether_rx) = mpsc::channel();
    let (quit_tether_tx, quit_tether_rx) = mpsc::channel();
    let tether_handle = start_tether_thread(tether_tx.clone(), quit_tether_rx);

    handles.push(tether_handle);

    let artnet = {
        if cli.artnet_broadcast {
            ArtNetInterface::new(ArtNetMode::Broadcast, cli.artnet_update_frequency)
        } else {
            ArtNetInterface::new(
                ArtNetMode::Unicast(
                    SocketAddr::from((cli.unicast_src, 6453)),
                    SocketAddr::from((cli.unicast_dst, 6454)),
                ),
                cli.artnet_update_frequency,
            )
        }
    };

    let mut model = Model::new(tether_rx, cli.clone(), artnet);

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
            if let Ok(_) = quit_cli_rx.try_recv() {
                info!("Headless loop should quit");
                should_quit = true;
            }
            std::thread::sleep(Duration::from_millis(1));
            model.update();
        }
        model.reset_before_quit();
    } else {
        info!("Running graphics mode; close the window to quit");
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(SIMPLE_WIN_SIZE),
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
    quit_tether_tx
        .send(())
        .expect("failed to send quit message via channel");
    for h in handles {
        match h.join() {
            Ok(()) => {
                debug!("Thread join OK");
            }
            Err(e) => {
                error!("Thread joined with error, {:?}", e);
            }
        }
    }
    std::thread::sleep(Duration::from_secs(1));
    info!("...Exit now");
    std::process::exit(0);
}
