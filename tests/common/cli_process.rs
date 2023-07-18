use std::io::{ErrorKind, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time;

use log::{error, trace};
use timeout_readwrite::TimeoutReader;

pub struct CliProcess {
    child_process: Child,
    output_reader: TimeoutReader<ChildStdout>,
    output_buffer: String,
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
                output_buffer: String::new(),
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("invalid command {}", command_str),
            ))
        }
    }

    pub fn poll_result(&mut self, do_wait: bool) -> Result<Option<i32>, std::io::Error> {
        self.read_output();
        match self.child_process.try_wait() {
            Ok(None) => {
                trace!("poll None");
                if do_wait {
                    trace!("waiting for exit..");
                    let status = self.child_process.wait()?;
                    trace!("exited with {}", status);
                    Ok(Some(status.code().unwrap()))
                } else {
                    Ok(None)
                }
            }
            Ok(Some(status)) => {
                trace!("poll {}", status);
                Ok(Some(status.code().unwrap()))
            }
            Err(e) => Err(e),
        }
    }

    fn read_output(&mut self) {
        match self.output_reader.read_to_string(&mut self.output_buffer) {
            Ok(read_bytes) => {
                trace!("read {} bytes", read_bytes);
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                trace!("read timed out");
            }
            Err(e) => error!("unexpected error {:?}", e),
        }
    }

    pub fn output_lines(&self) -> Vec<String> {
        self.output_buffer
            .lines()
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }
}
