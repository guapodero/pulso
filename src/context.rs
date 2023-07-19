use anyhow::Result;
use futures::stream::AbortHandle;
use log::info;

use crate::capture::{ExtractedHeaders, PacketOwned};

pub struct Context {
    pub device_name: String,
    pub count: usize,
}

impl Context {
    pub fn new(device_name: &str) -> Context {
        Context {
            device_name: device_name.to_string(),
            count: 0,
        }
    }

    pub fn process(&mut self, packet: PacketOwned, abort: &AbortHandle) -> Result<()> {
        self.count += 1;

        if self.count > 1 {
            info!("captured more than 1 packet, abort");
            let ExtractedHeaders {
                source_ip,
                dest_port,
                ..
            } = packet.headers()?;
            println!("{} {} {}", source_ip, dest_port, self.count);
            abort.abort();
        }

        Ok(())
    }
}
