use clap::Parser;

pub const UNICAST_SRC_STRING: &str = "127.0.0.1";
pub const UNICAST_DST_STRING: &str = "127.0.0.1";

pub const DEFAULT_ARTNET_HERTZ: u64 = 44;

pub const CHANNELS_PER_UNIVERSE: u16 = 512;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = "Tether Artnet Controller")]
pub struct Cli {
    /// Flag to enable headless (no GUI) mode, suitable for server-type
    /// process
    #[arg(long = "headless")]
    pub headless_mode: bool,

    #[arg(default_value_t=String::from("./example.project.json"))]
    pub project_path: String,

    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// Flag to enable ArtNet broadcast mode (good for development)
    #[arg(long = "artnet.broadcast")]
    pub artnet_broadcast: bool,

    /// Universe number for ArtNet, since there isn't a rigid standard for this
    #[arg(long = "artnet.universe", default_value_t = 1)]
    pub artnet_universe: u8,

    /// IP address for ArtNet source interface (ignored if broadcast enabled)
    #[arg(long = "artnet.interface")]
    pub unicast_src: Option<std::net::IpAddr>,

    /// IP address for ArtNet destination node (ignored if broadcast enabled)
    #[arg(long = "artnet.destination")]
    pub unicast_dst: Option<std::net::IpAddr>,

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

    /// Optionally set an ID/group for lighting-related Input Plugs (macros, scenes); useful for separating messages.
    /// Essentially defaults to wildcard (+) if omitted. Does NOT affect Tether MIDI subscriptions.
    #[arg(long = "tether.subscribe.id")]
    pub tether_subscribe_id: Option<String>,

    /// Host/IP for Tether MQTT Broker
    #[arg(long = "tether.host")]
    pub tether_host: Option<String>,
}
