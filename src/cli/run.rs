use clap::Parser;
use regex::Regex;

use super::Args;
use crate::{
    filter::{AndFilter, BoxedLogFilter, LevelFilter, MessageFilter, OrFilter, PidFilter, TagFilter, TidFilter},
    format::LogcatFormatter,
    input::LogcatInput,
    log::LogLine,
    parse::LogcatParser,
    prelude::*,
};

pub fn run() -> Result<()> {
    let args = Args::parse();
    let input_lines = select_input(&args)?.lines();
    let parser = LogcatParser::new();
    let filter = create_filter(&args)?;
    let formatter = LogcatFormatter::new();

    for line in input_lines {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let log_line = match parser.parse_log_line(&line) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("warning: {e}; see next line\n{line}");
                continue;
            },
        };

        if let LogLine::Entry(entry) = &log_line
            && !filter.include_entry(entry)
        {
            continue;
        }

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

fn create_filter(args: &Args) -> Result<BoxedLogFilter> {
    let mut filters: Vec<BoxedLogFilter> = Vec::new();

    if let Some(pid) = args.filter.pid {
        filters.push(Box::new(PidFilter(pid)));
    }

    if let Some(tid) = args.filter.tid {
        filters.push(Box::new(TidFilter(tid)));
    }

    if let Some(level) = args.filter.level {
        filters.push(Box::new(LevelFilter(level)));
    }

    if let Some(tags) = &args.filter.tag {
        let mut tag_filters = Vec::<BoxedLogFilter>::new();
        for tag in tags {
            tag_filters.push(Box::new(TagFilter::new(tag)?));
        }
        filters.push(Box::new(OrFilter(tag_filters)));
    }

    if let Some(messages) = &args.filter.message {
        let mut message_filters = Vec::<BoxedLogFilter>::new();
        for message in messages {
            message_filters.push(Box::new(MessageFilter(Regex::new(message)?)));
        }
        filters.push(Box::new(OrFilter(message_filters)));
    }

    Ok(Box::new(AndFilter(filters)))
}

fn is_piped() -> bool {
    !atty::is(atty::Stream::Stdin)
}
