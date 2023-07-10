use log::{debug, info};
use std::{thread, time};

use crate::common::{tcp_connect, tcp_listen, CliProcess};

mod common;

#[test]
fn test_count() {
    common::setup();
    tcp_listen("127.0.0.1:12345");

    let command = format!("{} --device lo", env!("CARGO_BIN_EXE_pulso"));
    debug!("command: {}", command);

    let mut process = CliProcess::new(&command).unwrap();

    thread::sleep(time::Duration::new(4, 250));
    // TODO wait here for PID file creation after stream start

    tcp_connect("127.0.0.1:12345");
    tcp_connect("127.0.0.1:12345");

    while let Ok(None) = process.poll_result() {
        info!("polling..");
        thread::sleep(time::Duration::new(4, 250));
    }

    info!("result: {:?}", process.result);

    assert_eq!(process.result.unwrap().std_out, vec!["127.0.0.1 12345 2"]);
}
