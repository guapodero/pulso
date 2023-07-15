use log::debug;
use std::{error, net};

use etherparse::{InternetSlice, SlicedPacket, TransportSlice};
use libc;
use pcap::{self, Active, Capture, Device, Direction, Packet, PacketCodec, PacketHeader};

use crate::context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOwned {
    capture_header: PacketHeader,
    data: Box<[u8]>,
}

#[derive(Debug)]
pub struct ExtractedHeaders {
    pub source_ip: net::IpAddr,
    pub dest_port: u16,
    pub capture_ts: libc::timeval,
}

impl PacketOwned {
    pub fn headers(&self) -> Result<ExtractedHeaders, std::io::Error> {
        match SlicedPacket::from_ethernet(&self.data) {
            Ok(SlicedPacket {
                ip: Some(InternetSlice::Ipv4(ip_headers, _)),
                transport: Some(TransportSlice::Tcp(tcp_headers)),
                ..
            }) => {
                let [a, b, c, d] = ip_headers.source();
                Ok(ExtractedHeaders {
                    source_ip: net::IpAddr::V4(net::Ipv4Addr::new(a, b, c, d)),
                    dest_port: tcp_headers.destination_port(),
                    capture_ts: self.capture_header.ts,
                })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "unparsed packet", // TODO more detail
            )),
        }
    }
}

pub struct Codec;

impl PacketCodec for Codec {
    type Item = PacketOwned;

    fn decode(&mut self, packet: Packet) -> Self::Item {
        debug!("decoding {:?}", packet);

        PacketOwned {
            capture_header: *packet.header,
            data: packet.data.into(),
        }
    }
}

pub fn capture_from_interface(
    context: &mut context::Context,
) -> Result<Capture<Active>, Box<dyn error::Error>> {
    let device = Device::list()?
        .into_iter()
        .find(|d| d.name == context.device_name)
        .unwrap();
    debug!("{:?}", device);

    let mut capture = Capture::from_device(device)?
        .snaplen(96)
        .immediate_mode(true)
        .open()?;

    // TODO capture both SYN and ACK parts of connection establishment
    // https://www.ietf.org/rfc/rfc9293.html#section-3.5
    // https://biot.com/capstats/bpf.html
    // https://wiki.wireshark.org/TCP_3_way_handshaking
    capture.direction(Direction::In)?;
    capture.filter(
        "tcp[tcpflags] & (tcp-syn) != 0 \
         and tcp[tcpflags] & (tcp-ack) = 0",
        true,
    )?;

    Ok(capture)
}
