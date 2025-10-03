use std::{
    fmt::Debug,
    fs::{self, File},
    io::{self, BufRead, BufReader, Lines, Stdin, StdinLock},
    process,
    process::Command,
};

use anyhow::anyhow;

use crate::prelude::*;

pub struct LogcatInput<B: BufRead> {
    lines: Lines<B>,
}

impl<B: BufRead> LogcatInput<B> {
    pub fn lines(self) -> impl Iterator<Item=Result<String>> {
        self.lines
            .map(|line| line.map_err(|e| anyhow!("failed to read line: {e}")))
    }
}

impl LogcatInput<BufReader<File>> {
    pub fn from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self { lines: reader.lines() })
    }
}

impl LogcatInput<StdinLock<'static>> {
    pub fn from_stdin() -> Self {
        Self {
            lines: io::stdin().lock().lines(),
        }
    }
}

impl LogcatInput<BufReader<process::ChildStdout>> {
    pub fn from_process(process: &mut process::Child) -> Result<Self> {
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow!("logcat child process doesn't have stdout"))?;

        Ok(Self {
            lines: BufReader::new(stdout).lines(),
        })
    }
}
