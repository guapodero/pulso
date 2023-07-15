use std::error;

use futures::stream::AbortHandle;
use log::info;

use crate::capture::{ExtractedHeaders, PacketOwned};

pub struct Context {
    pub device_name: String,
    pub count: usize,
}

impl Context {
    pub fn new(device_name: &str) -> Result<Context, Box<dyn error::Error>> {
        let context = Context {
            device_name: device_name.to_string(),
            count: 0,
        };

        Ok(context)
    }

    pub fn process(
        &mut self,
        packet: PacketOwned,
        abort: &AbortHandle,
    ) -> Result<(), Box<dyn error::Error>> {
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