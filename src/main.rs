use std::error;

use clap::Parser;
use log::debug;

use pulso::runtime;
use pulso::tcp;

/// TCP connection counter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// device name
    #[arg(short, long)]
    device: String,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();
    debug!("main");

    let args = Args::parse();

    let mut context = tcp::Context::new(&args.device)?;

    runtime::run_tokio_stream(&mut context)?;

    Ok(())
}
