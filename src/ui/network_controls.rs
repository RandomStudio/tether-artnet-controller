use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use egui::{Color32, RichText, Ui};
use log::debug;

use crate::{
    artnet::{ArtNetInterface, ArtNetMode},
    model::{attempt_connection, Model, TetherStatus},
    project::artnetconfig::ArtNetConfigMode,
    settings::{UNICAST_DST_STRING, UNICAST_SRC_STRING},
};
use anyhow::anyhow;

pub fn render_network_controls(model: &mut Model, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("Tether");

        match &model.tether_status {
            TetherStatus::NotConnected => {
                ui.label(RichText::new("Not (yet) connected").color(Color32::YELLOW));
                offer_tether_connect(model, ui);
            }
            TetherStatus::Connected => {
                ui.label(RichText::new("Connected").color(Color32::LIGHT_GREEN));
            }
            TetherStatus::Errored(msg) => {
                ui.label(RichText::new(msg).color(Color32::RED));
                offer_tether_connect(model, ui);
            }
        }
    });

    if let Some(artnet) = &model.artnet {
        let mut should_clear = false;
        ui.horizontal(|ui| {
            ui.heading("ArtNet");
            match artnet.mode_in_use() {
                ArtNetMode::Broadcast => {
                    ui.label(RichText::new("Broadcast Mode").color(Color32::LIGHT_YELLOW));
                }
                ArtNetMode::Unicast(src, dst) => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Unicast Mode: ").color(Color32::LIGHT_GREEN));
                        ui.small(format!("{} => {}", src, dst));
                    });
                }
            }
            if ui.button("✏").clicked() {
                debug!("Edit (and disable) ArtNet interface");
                // model.artnet = None;
                should_clear = true;
            }
        });
        if should_clear {
            model.artnet = None;
        }
    } else {
        ui.horizontal(|ui| {
            ui.heading("ArtNet");
            ui.label(RichText::new("Not connected").color(Color32::RED));
        });
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut model.artnet_edit_mode,
                ArtNetConfigMode::Broadcast,
                "Broadcast mode",
            );
            ui.radio_value(
                &mut model.artnet_edit_mode,
                ArtNetConfigMode::Unicast(UNICAST_SRC_STRING.into(), UNICAST_DST_STRING.into()),
                "Unicast mode",
            );
        });
        match &mut model.artnet_edit_mode {
            ArtNetConfigMode::Broadcast => (), // no settings for broadcast
            ArtNetConfigMode::Unicast(src, dst) => {
                ui.horizontal(|ui| {
                    ui.label("Network Interface IP");
                    ui.text_edit_singleline(src);
                });
                ui.horizontal(|ui| {
                    ui.label("Destination/ArtNet IP");
                    ui.text_edit_singleline(dst);
                });
            }
        }
        if ui.button("Apply & Connect").clicked() {
            let new_artnet_interface: Result<ArtNetInterface, anyhow::Error> =
                match &model.artnet_edit_mode {
                    ArtNetConfigMode::Broadcast => ArtNetInterface::new(
                        ArtNetMode::Broadcast,
                        model.settings.artnet_update_frequency,
                    ),
                    ArtNetConfigMode::Unicast(src, dst) => {
                        let src_parsed = Ipv4Addr::from_str(src);
                        let dst_parsed = Ipv4Addr::from_str(dst);
                        if src_parsed.is_ok() && dst_parsed.is_ok() {
                            ArtNetInterface::new(
                                ArtNetMode::Unicast(
                                    SocketAddr::from((Ipv4Addr::from_str(src).unwrap(), 6453)),
                                    SocketAddr::from((Ipv4Addr::from_str(dst).unwrap(), 6454)),
                                ),
                                model.settings.artnet_update_frequency,
                            )
                        } else {
                            Err(anyhow!("Invalid IP address string"))
                        }
                    }
                };
            if let Ok(interface) = new_artnet_interface {
                model.project.artnet_config = Some(ArtNetConfigMode::from(&interface));
                model.artnet = Some(interface);
            }
        }
    }
    ui.separator();
}

fn offer_tether_connect(model: &mut Model, ui: &mut Ui) {
    if ui.button("Connect").clicked() {
        attempt_connection(model);
    }
}
