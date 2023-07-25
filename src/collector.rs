use std::net::IpAddr;

use anyhow::Result;

use crate::capture::{ExtractedHeaders, PacketOwned};

#[derive(Default)]
pub struct Collector {
    connection_count: u64,
    last_ip: Option<IpAddr>,
    last_dest_port: Option<u16>,
}

impl Collector {
    pub fn process(&mut self, packet: PacketOwned) -> Result<u64> {
        let ExtractedHeaders {
            source_ip,
            dest_port,
            ..
        } = packet.headers()?;

        self.connection_count += 1;
        self.last_ip = Some(source_ip);
        self.last_dest_port = Some(dest_port);

        Ok(self.connection_count)
    }

    pub fn summary(&self) -> Option<String> {
        if self.connection_count > 0 {
            Some(format!(
                "{} {} {}",
                self.last_ip.unwrap(),
                self.last_dest_port.unwrap(),
                self.connection_count
            ))
        } else {
            None
        }
    }
}
