use std::io::{stdout, BufWriter};

use anyhow::Error;
use clap::Parser;
use color_print::cstr;
use log::{debug, error};

use pulso::collector::Collector;
use pulso::runtime::run_tokio_stream;

/// TCP connection counter
#[derive(Parser, Debug)]
#[command(author, version, about, after_help = AFTER_HELP)]
struct Args {
    /// device name
    #[arg(short, long)]
    device: String,
    /// max connections
    #[arg(short, long)]
    connection_limit: Option<u64>,
    /// max seconds
    #[arg(short, long)]
    time_limit: Option<u64>,
}

#[cfg(feature = "privacy")]
const AFTER_HELP: Option<&str> = Some(cstr!(
    r#"<bold><underline>Environment Variables:</underline></bold>
  PULSO_SECRET    (required) encryption key used for sensitive information
"#
));
#[cfg(not(feature = "privacy"))]
const AFTER_HELP: Option<&str> = None;

impl Args {
    fn parse() -> Self {
        let args = <Self as Parser>::parse();
        assert!(
            args.connection_limit
                .filter(|&l| l == 0)
                .or(args.time_limit.filter(|&l| l == 0))
                .is_none(),
            "limits must be positive",
        );

        #[cfg(feature = "privacy")]
        assert!(
            std::env::var("PULSO_SECRET").map_or(false, |s| !s.is_empty()),
            "Environment variable PULSO_SECRET must be non-empty"
        );

        args
    }
}

fn report_error(error: Error, msg: &str) {
    error!("{msg}: {:?}", error.root_cause());
    eprintln!(
        "{} {msg}\n\n\
            {error:?}\n\n\
            run with RUST_BACKTRACE=1 for further details",
        cstr!("<bold><red>error:</red></bold>")
    );
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
        report_error(e, "failed to start capture stream");
        std::process::exit(1);
    }

    debug!("stream finished. creating digest");

    let mut writer = BufWriter::new(stdout());

    if let Err(e) = collector.digest(&mut writer) {
        report_error(e, "failed to write digest output");
        std::process::exit(1);
    }
}
