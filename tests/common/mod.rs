use std::collections::HashMap;
use std::{net, thread, time};

use log::{error, info, trace};

use cli_process::CliProcess;

#[cfg(test)]
mod cli_process;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[derive(Debug)]
pub enum Output<T> {
    /// no output
    X,
    /// a specific value
    V(T),
    /// defer test
    F(fn(T)),
}

pub fn steps(steps: Vec<(&str, Output<i32>, Output<Vec<String>>)>) {
    let step_timeout = time::Duration::new(1, 500);

    let mut process: Option<CliProcess> = None;
    let mut envs: HashMap<String, String> = HashMap::new();
    let mut _listener: Option<thread::JoinHandle<()>> = None;
    let step_count = steps.len();

    for (i, (command, expect_exit, expect_stdout)) in steps.into_iter().enumerate() {
        info!("command: {}", command);
        match command {
            env if env.starts_with("env") => {
                let parts = env.split(&[' ', '=']).skip(1).collect::<Vec<&str>>();
                envs.insert(parts[0].to_string(), parts[1].to_string());
            }
            bin if bin.starts_with("$0") => {
                let bin_expanded = bin.replace("$0", env!("CARGO_BIN_EXE_pulso"));
                process = Some(CliProcess::new(&bin_expanded, &envs, step_timeout).unwrap());
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

        if process.is_none() {
            trace!("process not started yet, ignoring output. command: {}", command);
            continue;
        }

        let proc = process.as_mut().unwrap();
        let do_wait = i == step_count - 1;

        let result = proc.poll_result(do_wait).expect("poll failure");
        match expect_exit {
            Output::X => assert_eq!(result.is_none(), true, "expected no exit"),
            Output::V(expected) => assert_eq!(
                result.expect("expected exit"),
                expected,
                "command: {}",
                command
            ),
            Output::F(test) => test(result.expect("expected exit")),
        }

        let std_out = proc.output_lines();
        match expect_stdout {
            Output::X => {
                assert_eq!(std_out.is_empty(), true, "expected nothing from stdout")
            }
            Output::V(expected) => {
                assert_eq!(std_out, expected, "command: {}", command)
            }
            Output::F(test) => test(std_out),
        }
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
