use clap::Parser;
use colored::*;
use log::{debug, error};

use pulso::collector::Collector;
use pulso::runtime::run_tokio_stream;

/// TCP connection counter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// device name
    #[arg(short, long)]
    device: String,
    /// packet limit
    #[arg(short, long)]
    connection_limit: Option<u64>,
    /// time limit
    #[arg(short, long)]
    time_limit: Option<u64>,
}

fn main() {
    env_logger::init();
    debug!("main");

    let args = Args::parse();

    let mut collector = Collector::default();

    if let Err(e) = run_tokio_stream(
        &args.device,
        args.connection_limit,
        args.time_limit,
        &mut collector,
    ) {
        error!("failed to start capture stream: {:?}", e.root_cause());
        eprintln!(
            "{} failure during runtime creation:\n\
            {:?}\n\
            run with RUST_BACKTRACE=1 for further details",
            "error:".red().bold(),
            e
        );
        std::process::exit(1);
    }
}
