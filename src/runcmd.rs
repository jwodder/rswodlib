use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Stdio};
use std::str;
use thiserror::Error;

pub fn runcmd<I, S>(arg0: &str, args: I) -> Result<(), RuncmdError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let rc = Command::new(arg0)
        .args(args)
        .status()
        .map_err(RuncmdError::Startup)?;
    if rc.success() {
        Ok(())
    } else {
        Err(RuncmdError::Exit(rc))
    }
}

pub fn readcmd<I, S>(arg0: &str, args: I) -> Result<String, ReadcmdError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = Command::new(arg0)
        .args(args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(ReadcmdError::Startup)?;
    if out.status.success() {
        Ok(str::from_utf8(&out.stdout)
            .map_err(ReadcmdError::Decode)?
            .trim()
            .to_string())
    } else {
        Err(ReadcmdError::Exit(out.status))
    }
}

pub fn readcmd_lossy<I, S>(arg0: &str, args: I) -> Result<String, RuncmdError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = Command::new(arg0)
        .args(args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(RuncmdError::Startup)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(RuncmdError::Exit(out.status))
    }
}

#[derive(Debug, Error)]
pub enum RuncmdError {
    #[error("failed to execute command: {0}")]
    Startup(#[source] std::io::Error),
    #[error("command exited unsuccessfully: {0}")]
    Exit(ExitStatus),
}

#[derive(Debug, Error)]
pub enum ReadcmdError {
    #[error("failed to execute command: {0}")]
    Startup(#[source] std::io::Error),
    #[error("command exited unsuccessfully: {0}")]
    Exit(ExitStatus),
    #[error("could not decode command output: {0}")]
    Decode(#[source] str::Utf8Error),
}
