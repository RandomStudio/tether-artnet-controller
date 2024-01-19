use std::{
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::{Duration, SystemTime},
};

use artnet_protocol::{ArtCommand, Output};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    project::{CMYChannels, ChannelMacro, FixtureInstance, RGBWChannels},
    settings::CHANNELS_PER_UNIVERSE,
};

pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
    channels: Vec<u8>,
    update_interval: Duration,
    last_sent: Option<SystemTime>,
}

pub enum ArtNetMode {
    Broadcast,
    /// Specify from (interface) + to (destination) addresses
    Unicast(SocketAddr, SocketAddr),
}

impl ArtNetInterface {
    pub fn new(mode: ArtNetMode, update_frequency: u64) -> Self {
        let channels = Vec::with_capacity(CHANNELS_PER_UNIVERSE as usize);

        let update_interval = Duration::from_secs_f32(1.0 / update_frequency as f32);

        match mode {
            ArtNetMode::Broadcast => {
                let socket = UdpSocket::bind((String::from("0.0.0.0"), 6455)).unwrap();
                let broadcast_addr = ("255.255.255.255", 6454)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap();
                socket.set_broadcast(true).unwrap();
                ArtNetInterface {
                    socket,
                    destination: broadcast_addr,
                    channels,
                    update_interval,
                    last_sent: None,
                }
            }
            ArtNetMode::Unicast(src, destination) => {
                let socket = UdpSocket::bind(src).unwrap();

                socket.set_broadcast(false).unwrap();
                ArtNetInterface {
                    socket,
                    destination,
                    channels,
                    update_interval,
                    last_sent: None,
                }
            }
        }
    }

    pub fn update(
        &mut self,
        channels_state: &[u8],
        fixtures: &[FixtureInstance],
        apply_macros: bool,
    ) -> bool {
        match self.last_sent {
            Some(t) => {
                if t.elapsed().unwrap() < self.update_interval {
                    return false; // early return; not ready to send
                }
            }
            None => self.last_sent = Some(SystemTime::now()),
        }

        // zero(&mut self.channels);
        self.channels = channels_state.into(); // copy slice contents into Vec

        if apply_macros {
            for f in fixtures {
                for m in &f.config.active_mode.macros {
                    match m {
                        crate::project::FixtureMacro::Control(control_macro) => {
                            for c in &control_macro.channels {
                                self.channels[(*c - 1 + f.offset_channels) as usize] =
                                    control_macro.current_value;
                            }
                        }
                        crate::project::FixtureMacro::Colour(colour_macro) => {
                            match &colour_macro.channels {
                                crate::project::ChannelList::Additive(rgba) => {
                                    let RGBWChannels {
                                        red,
                                        green,
                                        blue,
                                        white,
                                    } = rgba;

                                    // Convert all rgb values from "opaque" version (ignoring alpha)
                                    let opaque = colour_macro.current_value.to_opaque();
                                    for c in red.iter() {
                                        self.channels[(*c - 1 + f.offset_channels) as usize] =
                                            opaque.r();
                                    }
                                    for c in green.iter() {
                                        self.channels[(*c - 1 + f.offset_channels) as usize] =
                                            opaque.g();
                                    }
                                    for c in blue.iter() {
                                        self.channels[(*c - 1 + f.offset_channels) as usize] =
                                            opaque.b();
                                    }

                                    // Use inverse of alpha for "white mix" , i.e.
                                    //  alpha = 100% => full saturation, no white
                                    //  alpha = 0% => RGB the same, but mix in full white
                                    let white_inverse = 255 - colour_macro.current_value.a();
                                    for c in white.iter() {
                                        self.channels[(*c - 1 + f.offset_channels) as usize] =
                                            white_inverse;
                                    }
                                }
                                crate::project::ChannelList::Subtractive(cmy) => {
                                    let CMYChannels {
                                        cyan,
                                        magenta,
                                        yellow,
                                        white,
                                    } = cmy;
                                    let brightness = colour_macro.current_value.a();

                                    let c = 255 - colour_macro.current_value.r();
                                    let m = 255 - colour_macro.current_value.g();
                                    let y = 255 - colour_macro.current_value.b();

                                    for channel in cyan.iter() {
                                        self.channels
                                            [(*channel - 1 + f.offset_channels) as usize] = c;
                                    }
                                    for channel in magenta.iter() {
                                        self.channels
                                            [(*channel - 1 + f.offset_channels) as usize] = m;
                                    }
                                    for channel in yellow.iter() {
                                        self.channels
                                            [(*channel - 1 + f.offset_channels) as usize] = y;
                                    }
                                    for channel in white.iter() {
                                        self.channels
                                            [(*channel - 1 + f.offset_channels) as usize] =
                                            brightness;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: self.channels.clone().into(), // make temp copy of self channel state (?)
            ..Output::default()
        });

        let buff = command.write_to_buffer().unwrap();
        self.socket.send_to(&buff, self.destination).unwrap();

        true
    }

    pub fn get_state(&self) -> &[u8] {
        &self.channels
    }
}

pub fn zero(channels: &mut Vec<u8>) {
    *channels = [0].repeat(CHANNELS_PER_UNIVERSE as usize);
}

pub fn random(channels: &mut Vec<u8>) {
    let mut rng = rand::thread_rng();
    for c in channels.iter_mut() {
        *c = rng.gen::<u8>();
    }
}
