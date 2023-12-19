use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Tether Artnet Controller")]
pub struct Cli {
  #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,
}