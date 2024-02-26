use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;

const UNICAST_SRC: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 102));
const UNICAST_DST: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

pub const DEFAULT_ARTNET_HERTZ: u64 = 44;

pub const CHANNELS_PER_UNIVERSE: u16 = 512;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = "Tether Artnet Controller")]
pub struct Cli {
    /// Flag to enable headless (no GUI) mode, suitable for server-type
    /// process
    #[arg(long = "headless")]
    pub headless_mode: bool,

    #[arg(long = "project",default_value_t=String::from("./example.project.json"))]
    pub project_path: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// Flag to enable ArtNet broadcast mode (good for development)
    #[arg(long = "artnet.broadcast")]
    pub artnet_broadcast: bool,

    /// IP address for ArtNet source interface (ignored if broadcast enabled)
    #[arg(long = "artnet.interface", default_value_t=UNICAST_SRC)]
    pub unicast_src: std::net::IpAddr,

    /// IP address for ArtNet destination node (ignored if broadcast enabled)
    #[arg(long = "artnet.destination", default_value_t=UNICAST_DST)]
    pub unicast_dst: std::net::IpAddr,

    /// Update frequency, in Hertz, for sending ArtNet data (gets converted to ms)
    #[arg(long = "artnet.freq", default_value_t=DEFAULT_ARTNET_HERTZ)]
    pub artnet_update_frequency: u64,

    // TODO: split tasks/commands such as "auto" into separate Clap Command
    #[arg(long = "auto.zero")]
    pub auto_zero: bool,

    #[arg(long = "auto.random")]
    pub auto_random: bool,

    /// Flag to disable Tether connect on start (GUI only)
    #[arg(long = "tether.noAutoConnect")]
    pub tether_disable_autoconnect: bool,
}
