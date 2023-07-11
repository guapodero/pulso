use std::io::{ErrorKind, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::{net, thread, time};

use log::{debug, error, info};
use timeout_readwrite::TimeoutReader;

pub fn setup() {
    env_logger::init();
}

pub struct CliProcess {
    child_process: Child,
    output_reader: TimeoutReader<ChildStdout>,
    pub output_lines: Vec<String>,
}

impl CliProcess {
    pub fn new(command_str: &str) -> Result<CliProcess, std::io::Error> {
        let parts = command_str.split(" ").collect::<Vec<&str>>();
        if let Some((command, args)) = parts.split_first() {
            let mut child = Command::new(command)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?;

            let stdout = child.stdout.take().unwrap();
            let reader = TimeoutReader::new(stdout, time::Duration::new(1, 0));

            Ok(CliProcess {
                child_process: child,
                output_reader: reader,
                output_lines: vec![],
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid command",
            ))
        }
    }

    pub fn poll_result(&mut self) -> Result<Option<i32>, std::io::Error> {
        match self.child_process.try_wait() {
            Ok(None) => {
                self.read_output();
                Ok(None)
            }
            Ok(Some(status)) => {
                self.read_output();
                Ok(Some(status.code().unwrap()))
            }
            Err(e) => Err(e),
        }
    }

    fn read_output(&mut self) {
        let mut buffer = String::new();
        match self.output_reader.read_to_string(&mut buffer) {
            Ok(read_bytes) => {
                if read_bytes > 0 {
                    self.output_lines.push(buffer.trim().to_owned());
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                self.output_lines.push("-".to_string());
            }
            Err(e) => error!("unexpected error {:?}", e),
        }
    }
}

pub fn tcp_listen<A: net::ToSocketAddrs>(addr: A) -> thread::JoinHandle<()> {
    debug!(
        "tcp_listen {}",
        addr.to_socket_addrs().unwrap().next().unwrap()
    );
    let listener = net::TcpListener::bind(addr).unwrap();
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

pub fn tcp_connect<A: net::ToSocketAddrs>(addr: A) {
    debug!(
        "tcp_connect {}",
        addr.to_socket_addrs().unwrap().next().unwrap()
    );
    {
        let stream = net::TcpStream::connect(addr).unwrap();
        info!("tcp_connect opened stream: {:?}", stream);
    }
    // connection closed here
}
