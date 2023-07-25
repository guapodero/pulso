use std::net::IpAddr;

use anyhow::Result;

use crate::capture::{ExtractedHeaders, PacketOwned};

pub struct Context {
    pub device_name: String,
    pub connection_limit: Option<usize>,
    pub time_limit: Option<u64>,
    connection_count: usize,
    last_ip: Option<IpAddr>,
    last_dest_port: Option<u16>,
}

impl Context {
    pub fn new(
        device_name: &str,
        connection_limit: Option<usize>,
        time_limit: Option<u64>,
    ) -> Context {
        Context {
            device_name: device_name.to_string(),
            connection_limit,
            time_limit,
            connection_count: 0,
            last_ip: None,
            last_dest_port: None,
        }
    }

    pub fn process(&mut self, packet: PacketOwned) -> Result<usize> {
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
