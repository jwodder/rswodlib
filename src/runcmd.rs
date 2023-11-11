use super::strings::trim_string::trim_string;
use bstr::ByteVec; // into_string_lossy()
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
        match String::from_utf8(out.stdout) {
            Ok(mut s) => {
                trim_string(&mut s);
                Ok(s)
            }
            Err(e) => Err(ReadcmdError::Decode(e.utf8_error())),
        }
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
        let mut s = <Vec<u8>>::into_string_lossy(out.stdout);
        trim_string(&mut s);
        Ok(s)
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;
    use predicates::prelude::*;

    #[cfg(unix)]
    #[test]
    fn runcmd_bad_exit() {
        let r = runcmd("sh", ["-c", "exit 42"]);
        let Err(RuncmdError::Exit(rc)) = r else {
            panic!("Command did not exit nonzero: {r:?}");
        };
        assert_eq!(rc.code(), Some(42));
    }

    #[cfg(unix)]
    #[test]
    fn runcmd_create_file() {
        let tmpfile = NamedTempFile::new("content.txt").unwrap();
        tmpfile.assert(predicate::path::missing());
        runcmd(
            "sh",
            [
                "-c",
                &format!(
                    "echo This is test text. > {}",
                    tmpfile.path().to_str().expect("Tempfile path is not UTF-8")
                ),
            ],
        )
        .unwrap();
        tmpfile.assert("This is test text.\n");
    }

    #[test]
    fn runcmd_startup_failure() {
        let r = runcmd::<[&str; 0], _>("this-command-does-not-exist", []);
        let Err(RuncmdError::Startup(_)) = r else {
            panic!("Command did not fail to start: {r:?}");
        };
    }

    #[cfg(unix)]
    #[test]
    fn test_readcmd() {
        let out = readcmd("printf", [r"  This text will be stripped.\n\n"]).unwrap();
        assert_eq!(out, "This text will be stripped.");
    }

    #[cfg(unix)]
    #[test]
    fn readcmd_non_utf8() {
        let r = readcmd("printf", [r"The byte \200 is not valid UTF-8.\n"]);
        let Err(ReadcmdError::Decode(_)) = r else {
            panic!("Command did not fail on decoding output: {r:?}");
        };
    }

    #[cfg(unix)]
    #[test]
    fn readcmd_bad_exit() {
        let r = readcmd("sh", ["-c", r"printf 'This will be discarded.\n'; exit 23"]);
        let Err(ReadcmdError::Exit(rc)) = r else {
            panic!("Command did not exit nonzero: {r:?}");
        };
        assert_eq!(rc.code(), Some(23));
    }

    #[test]
    fn readcmd_startup_failure() {
        let r = readcmd("nonexistent-echo", ["This", "is", "test", "text."]);
        let Err(ReadcmdError::Startup(_)) = r else {
            panic!("Command did not fail to start: {r:?}");
        };
    }

    #[cfg(unix)]
    #[test]
    fn test_readcmd_lossy() {
        let out = readcmd_lossy("printf", [r"  This text will be stripped.\n\n"]).unwrap();
        assert_eq!(out, "This text will be stripped.");
    }

    #[cfg(unix)]
    #[test]
    fn readcmd_lossy_non_utf8() {
        let out = readcmd_lossy("printf", [r"The byte \200 is not valid UTF-8.\n"]).unwrap();
        assert_eq!(out, "The byte \u{FFFD} is not valid UTF-8.");
    }

    #[cfg(unix)]
    #[test]
    fn readcmd_lossy_bad_exit() {
        let r = readcmd_lossy("sh", ["-c", r"printf 'This will be discarded.\n'; exit 23"]);
        let Err(RuncmdError::Exit(rc)) = r else {
            panic!("Command did not exit nonzero: {r:?}");
        };
        assert_eq!(rc.code(), Some(23));
    }

    #[test]
    fn readcmd_lossy_startup_failure() {
        let r = readcmd_lossy("nonexistent-echo", ["This", "is", "test", "text."]);
        let Err(RuncmdError::Startup(_)) = r else {
            panic!("Command did not fail to start: {r:?}");
        };
    }
}
