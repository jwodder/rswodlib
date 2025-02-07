use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

pub trait CommandExt {
    fn combined_output(&mut self) -> Result<(String, ExitStatus), CombinedOutputError>;
}

impl CommandExt for Command {
    fn combined_output(&mut self) -> Result<(String, ExitStatus), CombinedOutputError> {
        // <https://stackoverflow.com/a/72831067/744178>
        let mut child = self
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(CombinedOutputError::Startup)?;
        let child_stdout = child
            .stdout
            .take()
            .expect("child.stdout should be non-None");
        let child_stderr = child
            .stderr
            .take()
            .expect("child.stderr should be non-None");
        let (sender, receiver) = std::sync::mpsc::channel();
        let stdout_sender = sender.clone();
        let stdout_thread = std::thread::spawn(move || {
            let mut stdout = BufReader::new(child_stdout);
            loop {
                let mut line = String::new();
                if stdout.read_line(&mut line)? == 0 {
                    break;
                }
                if stdout_sender.send(line).is_err() {
                    break;
                }
            }
            Ok(())
        });
        let stderr_sender = sender.clone();
        let stderr_thread = std::thread::spawn(move || {
            let mut stderr = BufReader::new(child_stderr);
            loop {
                let mut line = String::new();
                if stderr.read_line(&mut line)? == 0 {
                    break;
                }
                if stderr_sender.send(line).is_err() {
                    break;
                }
            }
            Ok(())
        });
        drop(sender);
        let rc = child.wait().map_err(CombinedOutputError::Wait)?;
        match stdout_thread.join() {
            Ok(Ok(())) => (),
            Ok(Err(source)) => return Err(CombinedOutputError::Read(source)),
            Err(barf) => std::panic::resume_unwind(barf),
        }
        match stderr_thread.join() {
            Ok(Ok(())) => (),
            Ok(Err(source)) => return Err(CombinedOutputError::Read(source)),
            Err(barf) => std::panic::resume_unwind(barf),
        }
        let output = receiver.into_iter().collect::<String>();
        Ok((output, rc))
    }
}

#[derive(Debug, Error)]
pub enum CombinedOutputError {
    #[error("failed to run command")]
    Startup(#[source] std::io::Error),
    #[error("error reading output from command")]
    Read(#[source] std::io::Error),
    #[error("error waiting for command to terminate")]
    Wait(#[source] std::io::Error),
}
