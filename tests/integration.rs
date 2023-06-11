use log::{debug, error, info};
use std::{net, thread, time};

use crate::common::CliProcess;

mod common;

#[test]
fn test_count() {
    common::setup();
    tcp_listen();

    let command = format!("{} --device lo", env!("CARGO_BIN_EXE_pulso"));
    debug!("command: {}", command);

    let mut process = CliProcess::new(&command).unwrap();
    // todo comment about async polling implementation in rust
    process.poll_result().unwrap();
    thread::sleep(time::Duration::new(4, 250));
    // todo wait here for PID file creation after stream start

    tcp_connect();
    tcp_connect();

    while let Ok(None) = process.poll_result() {
        info!("polling..");
        thread::sleep(time::Duration::new(4, 250));
    }

    info!("result: {:?}", process.result);
}

fn tcp_listen() -> thread::JoinHandle<()> {
    debug!("tcp_listen");
    let listener = net::TcpListener::bind("127.0.0.1:12345").unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => info!("tcp_listen received connection {:?}", s),
                Err(e) => error!("tcp_listen encountered IO error: {:?}", e),
            }
        }
    })
    // socket will be closed when JoinHandle goes out of scope
}

fn tcp_connect() {
    debug!("tcp_connect");
    {
        let stream = net::TcpStream::connect("127.0.0.1:12345").unwrap();
        info!("tcp_connect opened stream: {:?}", stream);
    }
    // connection closed here
}
