use std::io::{ErrorKind, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time;

use log::{error, trace};
use timeout_readwrite::TimeoutReader;

pub struct CliProcess {
    child_process: Child,
    output_reader: TimeoutReader<ChildStdout>,
    pub last_output: Option<Vec<String>>,
}

impl CliProcess {
    pub fn new(
        command_str: &str,
        read_timeout: time::Duration,
    ) -> Result<CliProcess, std::io::Error> {
        let parts = command_str.split(" ").collect::<Vec<&str>>();
        if let Some((command, args)) = parts.split_first() {
            let mut child = Command::new(command)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?;

            let stdout = child.stdout.take().unwrap();
            let reader = TimeoutReader::new(stdout, read_timeout);

            Ok(CliProcess {
                child_process: child,
                output_reader: reader,
                last_output: None,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid command",
            ))
        }
    }

    pub fn poll_result(&mut self) -> Result<Option<i32>, std::io::Error> {
        self.read_output();
        match self.child_process.try_wait() {
            Ok(None) => {
                trace!("poll None");
                Ok(None)
            }
            Ok(Some(status)) => {
                trace!("poll {status}");
                Ok(Some(status.code().unwrap()))
            }
            Err(e) => Err(e),
        }
    }

    fn read_output(&mut self) {
        let mut buffer = String::new();
        match self.output_reader.read_to_string(&mut buffer) {
            Ok(read_bytes) => {
                trace!("read {read_bytes} bytes");
                if read_bytes > 0 {
                    let lines = buffer.trim().lines().map(|s| s.to_owned()).collect();
                    self.last_output = Some(lines);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                trace!("read timed out");
                self.last_output = None;
            }
            Err(e) => error!("unexpected error {:?}", e),
        }
    }
}
