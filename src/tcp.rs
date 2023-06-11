use log::info;
use std::error;

use futures::stream::AbortHandle;
use pcap::{self, Active, Capture, Device, Direction, Packet, PacketCodec, PacketHeader};

// todo deconstruct for processing using header format
// https://www.ietf.org/rfc/rfc9293.html#section-3.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOwned {
    pub header: PacketHeader,
    pub data: Box<[u8]>,
}

pub struct Codec;

impl PacketCodec for Codec {
    type Item = PacketOwned;

    fn decode(&mut self, packet: Packet) -> Self::Item {
        PacketOwned {
            header: *packet.header,
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
        println!("device: {:?}", device);

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

        // todo capture both SYN and ACK parts of connection establishment
        // https://www.ietf.org/rfc/rfc9293.html#section-3.5
        // https://biot.com/capstats/bpf.html
        capture.direction(Direction::In)?;
        capture.filter(
            "tcp[tcpflags] & (tcp-syn) != 0 \
             and tcp[tcpflags] & (tcp-ack) = 0",
            true,
        )?;

        Ok(capture)
    }

    pub fn process(&mut self, packet: PacketOwned) -> Result<(), Box<dyn error::Error>> {
        info!("captured {:?}", packet);

        self.count += 1;

        if self.count > 1 {
            if let Some(abort) = &self.abort {
                info!("captured more than 1 packet, abort");
                abort.abort();
            }
        }

        Ok(())
    }
}
