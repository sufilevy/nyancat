use std::{
    fs::File,
    io::{self, BufRead, BufReader, StdinLock},
    process,
    process::{Command, Stdio},
};

use anyhow::anyhow;
use bstr::{ByteSlice, io::BufReadExt};

use crate::prelude::*;

#[derive(Debug)]
pub enum LogcatInput {
    File(BufReader<File>),
    Stdin(StdinLock<'static>),
    Process(BufReader<process::ChildStdout>),
}

impl LogcatInput {
    pub fn lines(self) -> Box<dyn Iterator<Item = Result<String>>> {
        match self {
            Self::File(file) => Self::read_lines_from(file, "file"),
            Self::Stdin(stdin) => Self::read_lines_from(stdin, "stdin"),
            Self::Process(process) => Self::read_lines_from(process, "logcat process stdout"),
        }
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self::File(reader))
    }

    pub fn from_stdin() -> Self {
        Self::Stdin(io::stdin().lock())
    }

    pub fn from_process() -> Result<Self> {
        let mut process = Command::new("adb")
            .arg("logcat")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to execute `adb logcat`: {e}"))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow!("logcat child process doesn't have stdout"))?;

        Ok(Self::Process(BufReader::new(stdout)))
    }

    fn read_lines_from(
        input: impl BufRead + 'static,
        input_src: &'static str,
    ) -> Box<dyn Iterator<Item = Result<String>>> {
        let lines = input.byte_lines().map(move |line| match line {
            Ok(line) => Ok(line.to_str_lossy().into_owned()),
            Err(e) => Err(anyhow!("failed to read line from {input_src}: {e}")),
        });
        Box::new(lines)
    }
}
