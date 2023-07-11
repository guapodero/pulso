use log::{debug, error, info};

use crate::common::{tcp_connect, tcp_listen, CliProcess};

mod common;

#[test]
fn test_count() {
    common::setup();
    tcp_listen("127.0.0.1:12345");

    let command = format!("{} --device lo", env!("CARGO_BIN_EXE_pulso"));
    debug!("command: {}", command);

    let mut process = CliProcess::new(&command).unwrap();
    // TODO wait here for PID file creation after stream start

    loop {
        tcp_connect("127.0.0.1:12345");
        match process.poll_result() {
            Ok(None) => {
                info!("incomplete..");
            }
            Ok(Some(code)) => {
                info!("finished with exit code {}", code);
                break;
            }
            Err(e) => {
                error!("finished with error {:?}", e);
            }
        }
    }

    info!("result: {:?}", process.output_lines);

    assert_eq!(
        process.output_lines.pop(),
        Some("127.0.0.1 12345 2".to_string())
    );
}
