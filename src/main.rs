#![allow(dead_code)]

use anyhow::anyhow;

use crate::{format::LogcatFormatter, parse::LogcatParser, prelude::*};

mod filter;
mod format;
mod input;
mod log;
mod parse;
mod prelude;

fn main() -> Result<()> {
    let parser = LogcatParser::new();
    let formatter = LogcatFormatter::new();

    let full_input = include_str!("../input/logcat.txt");
    let sanity_input = include_str!("../input/sanity.txt");
    let stacktrace_input = include_str!("../input/stacktrace.txt");

    process_input("Full", full_input, &parser, &formatter)?;
    process_input("Stacktrace", stacktrace_input, &parser, &formatter)?;
    process_input("Sanity", sanity_input, &parser, &formatter)?;

    Ok(())
}

fn process_input(title: &str, input: &str, parser: &LogcatParser, formatter: &LogcatFormatter) -> Result<()> {
    println!("\n*** {title} ***\n");

    for line in input.lines() {
        let log_line = parser
            .parse_log_line(line)
            .map_err(|e| anyhow!("failed to parse log line: '{line}' with error: '{e}'"))?;

        let formatted_log_line = formatter.format_log_line(&log_line);
        println!("{formatted_log_line}");
    }

    Ok(())
}
