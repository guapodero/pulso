use std::{net, thread, time};

use log::{error, info, trace};

use cli_process::CliProcess;

mod cli_process;

pub fn steps(steps: Vec<(&str, Option<i32>, Option<Vec<&str>>)>, step_timeout: time::Duration) {
    env_logger::init();

    let owned_vec = |v: Vec<&str>| -> Vec<String> { v.into_iter().map(|s| s.to_owned()).collect() };

    let mut process: Option<CliProcess> = None;
    let mut _listener: Option<thread::JoinHandle<()>> = None;

    for (command, expect_exit, expect_lines) in steps {
        info!("command: {command}");
        match command {
            bin if bin.starts_with("$0") => {
                let bin_expanded = bin.replace("$0", env!("CARGO_BIN_EXE_pulso"));
                process = Some(CliProcess::new(&bin_expanded, step_timeout).unwrap());
            }
            listen if listen.starts_with("tcp_listen") => {
                if let Some(addr) = listen.split_whitespace().last() {
                    _listener = Some(tcp_listen(addr));
                } else {
                    error!("invalid command");
                }
            }
            connect if connect.starts_with("tcp_connect") => {
                if let Some(addr) = connect.split_whitespace().last() {
                    tcp_connect(addr);
                } else {
                    error!("invalid command");
                }
            }
            &_ => todo!(),
        }

        let proc = process.as_mut().unwrap();
        let result = proc.poll_result().expect("poll failure");

        assert_eq!(result, expect_exit, "command: {command}");
        assert_eq!(
            proc.last_output,
            expect_lines.map(owned_vec),
            "command: {command}"
        );
    }
}

pub fn tcp_listen<A: net::ToSocketAddrs>(addr: A) -> thread::JoinHandle<()> {
    let listener = net::TcpListener::bind(addr).unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => trace!("tcp_listen received connection {:?}", s),
                Err(e) => error!("tcp_listen encountered IO error: {:?}", e),
            }
        }
    })
    // socket will be closed when JoinHandle goes out of scope
}

pub fn tcp_connect<A: net::ToSocketAddrs>(addr: A) {
    let stream = net::TcpStream::connect(addr).unwrap();
    trace!("tcp_connect opened stream: {:?}", stream);
    // connection closed here
}
