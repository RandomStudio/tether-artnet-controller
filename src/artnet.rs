use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use artnet_protocol::{ArtCommand, Output};
use rand::{rngs::ThreadRng, Rng};

use crate::settings::CHANNELS_PER_UNIVERSE;

pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
    channels: Vec<u8>,
}

pub enum ArtNetMode {
    Broadcast,
    /// Specify from (interface) + to (destination) addresses
    Unicast(SocketAddr, SocketAddr),
}

impl ArtNetInterface {
    pub fn new(mode: ArtNetMode) -> Self {
        let channels = Vec::with_capacity(CHANNELS_PER_UNIVERSE as usize);

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
                }
            }
            ArtNetMode::Unicast(src, destination) => {
                let socket = UdpSocket::bind(src).unwrap();

                socket.set_broadcast(false).unwrap();
                ArtNetInterface {
                    socket,
                    destination,
                    channels,
                }
            }
        }
    }

    pub fn update(&mut self, channels_state: &[u8]) {
        zero(&mut self.channels);
        self.channels = channels_state.into(); // copy slice contents into Vec
        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: self.channels.clone().into(), // make temp copy of self channel state (?)
            ..Output::default()
        });

        let buff = command.write_to_buffer().unwrap();
        self.socket.send_to(&buff, self.destination).unwrap();
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
