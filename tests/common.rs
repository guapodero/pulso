use std::io::Read;
use std::process::{Child, Command, Stdio};

pub fn setup() {
    env_logger::init();
}

pub struct CliProcess {
    pub command: String,
    pub result: Option<CliResult>,
    child_process: Child,
}

#[derive(Debug)]
pub struct CliResult {
    pub status: i32,
    pub std_out: Vec<String>,
}

impl CliProcess {
    pub fn new(command_str: &str) -> Result<CliProcess, std::io::Error> {
        let parts = command_str.split(" ").collect::<Vec<&str>>();
        if let Some((command, args)) = parts.split_first() {
            let child = Command::new(command)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?;

            Ok(CliProcess {
                command: command_str.to_string(),
                result: None,
                child_process: child,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invlalid command",
            ))
        }
    }

    pub fn poll_result(&mut self) -> Result<Option<()>, std::io::Error> {
        match self.child_process.try_wait() {
            Ok(None) => Ok(None),
            Ok(Some(status)) => {
                let output: Vec<u8> = self
                    .child_process
                    .stdout
                    .take()
                    .unwrap()
                    .bytes()
                    .flatten()
                    .collect();
                let lines = String::from_utf8(output)
                    .unwrap()
                    .lines()
                    .map(|s| s.to_string())
                    .collect();
                self.result = Some(CliResult {
                    status: status.code().unwrap(),
                    std_out: lines,
                });
                Ok(Some(()))
            }
            Err(e) => Err(e),
        }
    }
}
