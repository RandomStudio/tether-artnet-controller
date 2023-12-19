use std::{net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr, ToSocketAddrs}, thread::sleep, time::Duration};

use artnet_protocol::{ArtCommand, Output};
use tether_agent::TetherAgentOptionsBuilder;
use rand::Rng;

pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
}
fn main() {
    let agent = TetherAgentOptionsBuilder::new("ArtnetController").build().expect("failed to init Tether Agent");

    let src = SocketAddr::from((Ipv4Addr::new(10, 112, 10, 187), 6453));
    let dst = SocketAddr::from((Ipv4Addr::new(10,112,10,187), 6454));

    let socket = UdpSocket::bind(src).unwrap();

    let artnet = ArtNetInterface{
        socket,
        destination: dst
    };

    let mut rng = rand::thread_rng();

    loop {
        let mut channels: Vec<u8> = Vec::with_capacity(64);

        for _i in 0..64 {
            channels.push(rng.gen::<u8>());
        }



        sleep(Duration::from_millis(1000));

        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: channels.into(),
            ..Output::default()
        });
        let buff = command.write_to_buffer().unwrap();
        artnet.socket.send_to(&buff, artnet.destination).unwrap();
    }

}
