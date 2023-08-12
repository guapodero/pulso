use anyhow::{Context, Result};
use futures::stream::StreamExt;
use log::{debug, error, info, warn};
use tokio::runtime::{self, Runtime as TokioRuntime};
use tokio::time::{self, Duration};

use crate::capture::{capture_from_device, Codec};
use crate::collector::Collector;

pub fn collect_async(
    device_name: &str,
    connection_limit: Option<u64>,
    time_limit: Option<u64>,
    collector: &mut Collector,
) -> Result<()> {
    let capture = capture_from_device(device_name)?.setnonblock()?;
    let timeout_duration = Duration::from_secs(time_limit.unwrap_or(u64::MAX));

    debug!("starting tokio runtime");
    let runtime: TokioRuntime = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .context("build tokio runtime")?;

    runtime.block_on(async {
        let mut stream = capture
            .stream(Codec)
            .context("capture from interface as stream")?;

        let timeout_future = time::timeout(timeout_duration, futures::future::pending::<()>());
        tokio::pin!(timeout_future);

        info!("starting capture on device: {}", device_name);

        loop {
            tokio::select! {
                next = stream.next() => match next {
                   Some(Ok(packet)) => match (collector.process(packet), connection_limit) {
                        (Ok(count), Some(limit)) if count > limit - 1 => {
                            info!("connection limit reached. exiting");
                            break;
                        }
                        (Err(e), _) => warn!("processing error: {:#}", e),
                        _ => (),
                    },
                    Some(Err(pcap_error)) => error!("capture error: {:?}", pcap_error),
                    None => {
                        warn!("capture stream closed. exiting");
                        break;
                    }
                },
                _ = &mut timeout_future => {
                    info!("time limit reached. exiting");
                    break;
                }
                // TODO break on SIGINT
            }
        }

        info!("pcap stats {:?}", stream.capture_mut().stats());

        Ok(())
    })
}
