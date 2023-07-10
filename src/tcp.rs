use log::{debug, info};
use std::{error, net};

use etherparse::{InternetSlice, SlicedPacket, TransportSlice};
use futures::stream::AbortHandle;
use libc;
use pcap::{self, Active, Capture, Device, Direction, Packet, PacketCodec, PacketHeader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOwned {
    capture_header: PacketHeader,
    data: Box<[u8]>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PacketHeaders {
    source_ip: net::IpAddr,
    dest_port: u16,
    capture_ts: libc::timeval,
}

impl PacketOwned {
    pub fn headers(&self) -> Result<PacketHeaders, std::io::Error> {
        match SlicedPacket::from_ethernet(&self.data) {
            Ok(SlicedPacket {
                ip: Some(InternetSlice::Ipv4(ip_headers, _)),
                transport: Some(TransportSlice::Tcp(tcp_headers)),
                ..
            }) => {
                let [a, b, c, d] = ip_headers.source();
                Ok(PacketHeaders {
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

pub struct Context {
    pub device: Device,
    abort: Option<AbortHandle>,
    count: usize,
}

impl Context {
    pub fn new(device_name: &str) -> Result<Context, Box<dyn error::Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == device_name)
            .unwrap();
        debug!("{:?}", device);

        let context = Context {
            device,
            abort: None,
            count: 0,
        };

        Ok(context)
    }

    pub fn set_abort(&mut self, abort: AbortHandle) {
        self.abort = Some(abort);
    }

    pub fn capture(&self) -> Result<Capture<Active>, Box<dyn error::Error>> {
        let mut capture = Capture::from_device(self.device.clone())?
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

    pub fn process(&mut self, packet: PacketOwned) -> Result<(), Box<dyn error::Error>> {
        self.count += 1;

        if self.count > 1 {
            if let Some(abort) = &self.abort {
                info!("captured more than 1 packet, abort");
                let PacketHeaders {
                    source_ip,
                    dest_port,
                    ..
                } = packet.headers()?;
                println!("{} {} {}", source_ip, dest_port, self.count);
                abort.abort();
            }
        }

        Ok(())
    }
}
