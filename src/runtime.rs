use anyhow::{anyhow, Context as AContext, Result};
use futures::stream::{abortable, AbortHandle, StreamExt};
use log::{debug, error, info};
use tokio::runtime::{self, Runtime as TokioRuntime};
use tokio::time::{timeout, Duration};

use crate::capture::{capture_from_interface, Codec, PacketOwned};
use crate::context::Context;

pub fn run_tokio_stream(context: &mut Context) -> Result<()> {
    debug!("starting tokio runtime");

    let runtime: TokioRuntime = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .context("build tokio runtime")?;

    let capture = capture_from_interface(context)?;

    let mut abort_wrapper: Option<AbortHandle> = None;

    let stream = runtime.block_on(async {
        let stream = capture
            .setnonblock()
            .unwrap()
            .stream(Codec)
            .expect("capture from interface as stream");
        let (abortable, abort) = abortable(stream);
        abort_wrapper = Some(abort);
        abortable
    });

    let abort = abort_wrapper.ok_or(anyhow!("create stream abort handle"))?;
    let time_limit = context.time_limit.take();

    let finish = stream.for_each(|next: Result<PacketOwned, pcap::Error>| {
        match next {
            Ok(packet) => match (context.process(packet), context.connection_limit) {
                (Ok(count), Some(limit)) if count > limit - 1 => {
                    info!("connection limit reached. exiting");
                    abort.abort();
                }
                (Err(e), _) => error!("processing error: {:?}", e),
                _ => (),
            },
            Err(pcap_error) => error!("capture error: {:?}", pcap_error),
        }
        futures::future::ready(())
    });
    info!("capture stream started");

    if let Some(max_seconds) = time_limit {
        let timeout_duration = Duration::from_secs(max_seconds);
        debug!("time limit set to {:?}", timeout_duration);
        let result = runtime.block_on(async { timeout(timeout_duration, finish).await });
        if result.is_err() {
            info!("time limit reached. exiting");
        }
    } else {
        runtime.block_on(finish);
    }
    debug!("stream finished");

    if let Some(summary) = context.summary() {
        println!("{}", summary);
    }

    Ok(())
}
