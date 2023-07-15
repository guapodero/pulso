use log::{debug, error};
use std::error;

use futures::stream::{abortable, AbortHandle, StreamExt};
use tokio::runtime::{self, Runtime as TokioRuntime};

use crate::capture::{capture_from_interface, Codec, PacketOwned};
use crate::context::Context;

pub fn run_tokio_stream(context: &mut Context) -> Result<(), Box<dyn error::Error>> {
    debug!("starting tokio runtime");

    let runtime: TokioRuntime = runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    let mut abort_wrapper: Option<AbortHandle> = None;

    let stream = runtime.block_on(async {
        let capture = capture_from_interface(context).unwrap();
        let stream = capture.setnonblock().unwrap().stream(Codec).unwrap();
        let (abortable, abort) = abortable(stream);
        abort_wrapper = Some(abort);
        abortable
    });

    let abort = abort_wrapper.unwrap();

    let finish = stream.for_each(move |next: Result<PacketOwned, pcap::Error>| {
        match next {
            Ok(packet) => {
                if let Err(e) = context.process(packet, &abort) {
                    error!("processing error: {:?}", e);
                }
            }
            Err(pcap_error) => error!("capture error: {:?}", pcap_error),
        }
        futures::future::ready(())
    });
    debug!("waiting for end of stream");

    runtime.block_on(finish);
    debug!("stream finished");

    Ok(())
}
