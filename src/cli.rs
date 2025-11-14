use anyhow::anyhow;
use clap::{Parser, ValueHint};

use crate::{format::LogcatFormatter, input::LogcatInput, parse::LogcatParser, prelude::*};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct Args {
    /// Read input from stdin. [default when input is piped into the program]
    #[arg(long, conflicts_with_all(["input_file", "exec_adb_logcat"]), default_value_t = false)]
    stdin: bool,

    #[clap(flatten)]
    input: Input,
}

#[derive(Debug, Parser)]
#[group(required(false), multiple(false))]
pub struct Input {
    /// Path to an input file to read and process.
    #[arg(long("file"), value_name("FILE"), value_hint(ValueHint::FilePath))]
    input_file: Option<String>,

    /// Execute `adb logcat`, capture its output and process it. [default when no input is piped into the program]
    #[arg(long("exec-logcat"), default_value_t = false)]
    exec_adb_logcat: bool,
}

pub fn run_cli() -> Result<()> {
    let args = Args::parse();
    let input_lines = select_input(&args)?.lines();
    let parser = LogcatParser::new();
    let formatter = LogcatFormatter::new();

    for line in input_lines {
        let line = line?;
        let log_line = parser
            .parse_log_line(&line)
            .map_err(|e| anyhow!("{e}: '{line}'"))?;

        let formatted_log_line = formatter.format_log_line(&log_line);
        println!("{formatted_log_line}");
    }

    Ok(())
}

fn select_input(args: &Args) -> Result<LogcatInput> {
    if let Some(input_file) = &args.input.input_file {
        return LogcatInput::from_file(input_file);
    }

    if args.input.exec_adb_logcat {
        return LogcatInput::from_process();
    }

    if is_piped() {
        return Ok(LogcatInput::from_stdin());
    } else if args.stdin {
        eprintln!("warning: stdin flag is provided but no input is piped into the program");
        return Ok(LogcatInput::from_stdin());
    }

    LogcatInput::from_process()
}

fn is_piped() -> bool {
    !atty::is(atty::Stream::Stdin)
}
