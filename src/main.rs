#![allow(unused)]

use regex::Regex;

use crate::{
    filters::{AndFilter, LogFilter, LogLevelFilter, MessageFilter, TagFilter},
    input::LogcatInput,
    log::{LogLevel, LogLine},
    prelude::*,
};

mod filters;
mod input;
mod log;
mod parsing;
mod prelude;

fn main() -> Result<()> {
    let mut input = LogcatInput::from_file("logcat.txt")?;
    for line in input.lines() {
        let line = line?;
        let result = parsing::parse_log_line(&line).inspect_err(|e| {
            println!("Failed to parse log line: {:?}", line);
        })?;
        match result {
            LogLine::Header(header) => {
                println!("------- {header}")
            },
            LogLine::Entry(entry) => {
                println!("{} {} {}", entry.level, entry.tag, entry.message)
            },
        }
    }
    Ok(())
}
