use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use artnet_protocol::{ArtCommand, Output};
use log::*;
use rand::Rng;

use crate::{
    project::fixture::{
        CMYChannels, ChannelList, ChannelWithResolution, FixtureInstance, FixtureMacro,
        RGBWChannels,
    },
    settings::CHANNELS_PER_UNIVERSE,
};

pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
    channels: Vec<u8>,
    update_interval: Duration,
    last_sent: Option<SystemTime>,
    mode_in_use: ArtNetMode,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ArtNetMode {
    Broadcast,
    /// Specify from (interface) + to (destination) addresses
    Unicast(SocketAddr, SocketAddr),
}

impl ArtNetInterface {
    pub fn new(mode: ArtNetMode, update_frequency: u64) -> anyhow::Result<Self> {
        let channels = Vec::with_capacity(CHANNELS_PER_UNIVERSE as usize);

        let update_interval = Duration::from_secs_f32(1.0 / update_frequency as f32);

        match mode {
            ArtNetMode::Broadcast => {
                let socket = UdpSocket::bind((String::from("0.0.0.0"), 6455))?;
                let broadcast_addr = ("255.255.255.255", 6454).to_socket_addrs()?.next().unwrap();
                socket.set_broadcast(true).unwrap();
                debug!("Broadcast mode set up OK");
                Ok(ArtNetInterface {
                    socket,
                    destination: broadcast_addr,
                    channels,
                    update_interval,
                    last_sent: None,
                    mode_in_use: mode.clone(),
                })
            }
            ArtNetMode::Unicast(src, destination) => {
                info!(
                    "Will connect from interface {} to destination {}",
                    &src, &destination
                );
                match UdpSocket::bind(src) {
                    Ok(socket) => {
                        socket.set_broadcast(false)?;
                        Ok(ArtNetInterface {
                            socket,
                            destination,
                            channels,
                            update_interval,
                            last_sent: None,
                            mode_in_use: mode.clone(),
                        })
                    }
                    Err(e) => Err(anyhow!("Error binding socket: {}", e)),
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
                        FixtureMacro::Control(control_macro) => {
                            for c in &control_macro.channels {
                                match c {
                                    ChannelWithResolution::LoRes(single_channel) => {
                                        let target_channel =
                                            (*single_channel - 1 + f.offset_channels) as usize;
                                        let scaled_value = ((control_macro.current_value as f32
                                            / u16::MAX as f32)
                                            * 255.0)
                                            as u8;
                                        debug!(
                                            "Apply LoRes value to single fixture macro (channel {}) => {}, value {} => {}",
                                            single_channel,
                                            target_channel,
                                            control_macro.current_value,
                                            scaled_value
                                        );
                                        self.channels[target_channel] = scaled_value;
                                    }
                                    ChannelWithResolution::HiRes((c1, c2)) => {
                                        // Assume coarse+fine 16-bit values are "big endian" (be):
                                        let [b1, b2] = control_macro.current_value.to_be_bytes();
                                        // coarse channel:
                                        self.channels[(*c1 - 1 + f.offset_channels) as usize] = b1;
                                        // fine channel:
                                        self.channels[(*c2 - 1 + f.offset_channels) as usize] = b2;
                                    }
                                }
                            }
                        }
                        FixtureMacro::Colour(colour_macro) => {
                            match &colour_macro.channels {
                                ChannelList::Additive(rgba) => {
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
                                ChannelList::Subtractive(cmy) => {
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

        trace!("Channel state {:?}", self.channels);
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

    pub fn mode_in_use(&self) -> &ArtNetMode {
        &self.mode_in_use
    }
}

pub fn zero(channels: &mut Vec<u8>) {
    *channels = [0].repeat(CHANNELS_PER_UNIVERSE as usize);
}

pub fn random(channels: &mut [u8]) {
    let mut rng = rand::thread_rng();
    for c in channels.iter_mut() {
        *c = rng.gen::<u8>();
    }
}
