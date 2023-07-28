use clap::Parser;
use color_print::cstr;
use log::{debug, error};

use pulso::collector::Collector;
use pulso::runtime::run_tokio_stream;

const AFTER_HELP: &str = cstr!(
    r#"<bold><underline>Environment Variables:</underline></bold>
  PULSO_SECRET    (required) encryption key used for sensitive information
"#
);

/// TCP connection counter
#[derive(Parser, Debug)]
#[command(author, version, about, after_help = Some(AFTER_HELP))]
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
    assert!(
        std::env::var("PULSO_SECRET").map_or(false, |s| !s.is_empty()),
        "Environment variable PULSO_SECRET must be non-empty"
    );

    let mut collector = Collector::default();

    if let Err(e) = run_tokio_stream(
        &args.device,
        args.connection_limit,
        args.time_limit,
        &mut collector,
    ) {
        error!("failed to start capture stream: {:?}", e.root_cause());
        eprintln!(
            "{} failure during runtime creation\n\n\
            {:?}\n\n\
            run with RUST_BACKTRACE=1 for further details",
            cstr!("<bold><red>error:</red></bold>"),
            e
        );
        std::process::exit(1);
    }
}
