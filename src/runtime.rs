use log::{debug, error};
use std::error;

use futures::stream::{abortable, StreamExt};
use tokio::runtime::{self, Runtime as TokioRuntime};

use crate::tcp;

pub fn run_tokio_stream(context: &mut tcp::Context) -> Result<(), Box<dyn error::Error>> {
    debug!("starting tokio runtime");

    let runtime: TokioRuntime = runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    let stream = runtime.block_on(async {
        let capture_stream = context
            .capture()
            .unwrap()
            .setnonblock()
            .unwrap()
            .stream(tcp::Codec)
            .unwrap();
        let (abortable_stream, abort_handle) = abortable(capture_stream);
        context.set_abort(abort_handle);
        abortable_stream
    });

    let finish = stream.for_each(move |next: Result<tcp::PacketOwned, pcap::Error>| {
        match next {
            Ok(packet) => {
                if let Err(e) = context.process(packet) {
                    error!("processing error: {:?}", e);
                }
            }
            Err(pcap_error) => error!("capture error: {:?}", pcap_error),
        }
        futures::future::ready(())
    });
    debug!("waiting for end of stream");
    // todo write current process ID to a file to signal readiness for packets

    runtime.block_on(finish);
    debug!("stream finished");

    Ok(())
}
