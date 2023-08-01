use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use log::{error, trace};

use crate::common::cli_process::CliProcess;

pub struct Active;
pub struct Inactive;

pub trait Status {}
impl Status for Active {}
impl Status for Inactive {}

pub struct Scenario<T: Status> {
    process: Option<CliProcess>,
    env: HashMap<String, String>,
    step_timeout: Duration,
    binary_path: PathBuf,
    start_time: Option<Instant>,
    _listeners: Vec<thread::JoinHandle<()>>,
    _marker: PhantomData<T>,
}

impl Scenario<Inactive> {
    pub fn new<P: AsRef<Path>>(binary: P, step_timeout: Duration) -> Self {
        Scenario {
            process: None,
            env: HashMap::new(),
            step_timeout,
            binary_path: binary.as_ref().to_path_buf(),
            start_time: None,
            _listeners: vec![],
            _marker: PhantomData,
        }
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn start(self, args: &str) -> Scenario<Active> {
        let bin_expanded = format!("{} {}", self.binary_path.as_path().display(), args);
        Scenario {
            process: Some(CliProcess::new(&bin_expanded, &self.env, self.step_timeout).unwrap()),
            env: self.env,
            step_timeout: self.step_timeout,
            binary_path: self.binary_path,
            start_time: Some(Instant::now()),
            _listeners: self._listeners,
            _marker: PhantomData,
        }
    }
}

impl Scenario<Active> {
    pub fn check_result(mut self, exit: Option<i32>, stdout: impl Fn(Vec<&str>)) -> Self {
        let proc = self.process.as_mut().unwrap();
        let do_wait = exit.is_some();
        let result = proc.poll_result(do_wait).expect("poll failure");
        let output = proc.output_lines();
        assert_eq!(result, exit);
        stdout(output);
        self
    }

    pub fn check_duration(self, duration: fn(Duration)) -> Self {
        duration(self.start_time.unwrap().elapsed());
        self
    }

    pub fn tcp_listen<A: ToSocketAddrs>(mut self, addr: A) -> Self {
        let listener = TcpListener::bind(addr).unwrap();
        self._listeners.push(thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => trace!("tcp_listen received connection {:?}", s),
                    Err(e) => error!("tcp_listen encountered IO error: {:?}", e),
                }
            }
        }));
        self
    }

    pub fn tcp_connect<A: ToSocketAddrs>(self, addr: A) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        trace!("tcp_connect opened stream: {:?}", stream);
        self
    }
}
