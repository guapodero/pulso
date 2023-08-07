use anyhow::{anyhow, Context, Result};
use etherparse::{InternetSlice, SlicedPacket, TransportSlice};
use libc::timeval;
use log::debug;
use pcap::{self, Active, Capture, Device, Direction, Packet, PacketCodec, PacketHeader};

use crate::sensitive::IpAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOwned {
    pub capture_header: PacketHeader,
    data: Box<[u8]>,
}

#[derive(Debug)]
pub struct ExtractedHeaders {
    pub source_ip: IpAddress,
    pub dest_port: u16,
    pub capture_ts: timeval,
}

impl PacketOwned {
    pub fn headers(&self) -> Result<ExtractedHeaders> {
        match SlicedPacket::from_ethernet(&self.data) {
            Ok(SlicedPacket {
                ip: Some(ip_headers),
                transport: Some(TransportSlice::Tcp(tcp_headers)),
                ..
            }) => {
                let source_ip = match ip_headers {
                    InternetSlice::Ipv6(headers, _) => IpAddress::V6(headers.source()),
                    InternetSlice::Ipv4(headers, _) => IpAddress::V4(headers.source()),
                };
                Ok(ExtractedHeaders {
                    source_ip,
                    dest_port: tcp_headers.destination_port(),
                    capture_ts: self.capture_header.ts,
                })
            }
            Ok(skipped) => Err(anyhow!(
                "skipped {:?} at {:?}",
                skipped,
                self.capture_header.ts,
            )),
            Err(e) => Err(anyhow!(
                "parse failure {:?} at {:?}",
                e,
                self.capture_header.ts,
            )),
        }
    }
}

pub struct Codec;

impl PacketCodec for Codec {
    type Item = PacketOwned;

    fn decode(&mut self, packet: Packet) -> Self::Item {
        PacketOwned {
            capture_header: *packet.header,
            data: packet.data.into(),
        }
    }
}

pub fn capture_from_device(device_name: &str) -> Result<Capture<Active>> {
    let device = Device::list()
        .context("list devices")?
        .into_iter()
        .find(|d| d.name == device_name)
        .ok_or(anyhow!("device not found: {}", device_name))?;
    debug!("{:?}", device);

    let mut capture = Capture::from_device(device)
        .context("capture from device")?
        .snaplen(96)
        .immediate_mode(true)
        .open()
        .context("start capture")?;

    /*
    The full connection sequence looks like this:
    13:43:45.070344 IP localhost.34644 > localhost.italk: Flags [S],
    13:43:45.070361 IP localhost.italk > localhost.34644: Flags [S.],
    13:43:45.070377 IP localhost.34644 > localhost.italk: Flags [.],
    13:43:45.070420 IP localhost.34644 > localhost.italk: Flags [P.],
    13:43:45.070425 IP localhost.italk > localhost.34644: Flags [.],
    13:43:45.070560 IP localhost.34644 > localhost.italk: Flags [F.],
    13:43:45.070668 IP localhost.italk > localhost.34644: Flags [F.],
    13:43:45.070684 IP localhost.34644 > localhost.italk: Flags [.],
    Capturing only the first packet for now (SYN & !ACK)
    https://wiki.wireshark.org/TCP_3_way_handshaking
    https://www.ietf.org/rfc/rfc9293.html#section-3.5
    */
    capture
        .direction(Direction::In)
        .context("set capture direction")?;
    capture
        .filter(
            "(ip6 and proto \\tcp and ip6[40+13]&0x2 != 0 and ip6[40+13]&0x10 = 0) \
            or (ip and tcp[tcpflags] & (tcp-syn) != 0 and tcp[tcpflags] & (tcp-ack) = 0)",
            true,
        )
        .context("set capture filter")?;

    Ok(capture)
}
