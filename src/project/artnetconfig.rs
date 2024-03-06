use anyhow::anyhow;
use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    artnet::{ArtNetInterface, ArtNetMode},
    settings::Cli,
};

use super::Project;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ArtNetConfigMode {
    Broadcast,
    Unicast(String, String),
}

pub fn get_artnet_interface(
    cli: &Cli,
    project: &Project,
) -> Result<ArtNetInterface, anyhow::Error> {
    debug!("get_artnet_interface");
    if cli.artnet_broadcast {
        warn!("CLI artnetBroadcast flag overrides any Project ArtNet settings");
        ArtNetInterface::new(ArtNetMode::Broadcast, cli.artnet_update_frequency)
    } else if cli.unicast_src.is_some() && cli.unicast_dst.is_some() {
        warn!("CLI unicastSrc + unicastDst options override any Project ArtNet settings");
        ArtNetInterface::new(
            ArtNetMode::Unicast(
                SocketAddr::from((cli.unicast_src.unwrap(), 6453)),
                SocketAddr::from((cli.unicast_dst.unwrap(), 6454)),
            ),
            cli.artnet_update_frequency,
        )
    } else {
        debug!("No CLI overrides, attempt to use Project ArtNet config...");
        match &project.artnet_config {
            Some(artnet_mode) => {
                info!("Using project ArtNet Config {:?}", artnet_mode);
                match artnet_mode {
                    ArtNetConfigMode::Broadcast => {
                        ArtNetInterface::new(ArtNetMode::Broadcast, cli.artnet_update_frequency)
                    }
                    ArtNetConfigMode::Unicast(interface_ip, destination_ip) => {
                        ArtNetInterface::new(
                            ArtNetMode::Unicast(
                                SocketAddr::from((Ipv4Addr::from_str(interface_ip).unwrap(), 6453)),
                                SocketAddr::from((
                                    Ipv4Addr::from_str(destination_ip).unwrap(),
                                    6454,
                                )),
                            ),
                            cli.artnet_update_frequency,
                        )
                    }
                }
            }
            None => {
                error!("ArtNet config could not be found or parsed from Project");
                Err(anyhow!(
                    "No artnet settings in Project or CLI; user should connect manually"
                ))
            }
        }
    }
}

impl From<&ArtNetInterface> for ArtNetConfigMode {
    fn from(value: &ArtNetInterface) -> Self {
        match value.mode_in_use() {
            ArtNetMode::Broadcast => ArtNetConfigMode::Broadcast,
            ArtNetMode::Unicast(src, dst) => {
                ArtNetConfigMode::Unicast(src.ip().to_string(), dst.ip().to_string())
            }
        }
    }
}
