use std::cell::Cell;

use colored::{Color, ColoredString, Colorize};
use lazy_regex::{Lazy, regex};
use nonempty_collections::nev;
use regex::Regex;
use time::{UtcDateTime, format_description::BorrowedFormatItem};
use time_macros::format_description;

use super::{colors, log_line::FormattedLogLine};
use crate::{
    log::{LogEntry, LogLevel, LogLine},
    parse::MISSING_TAG,
};

const DATETIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");

const MAX_TAG_LENGTH: usize = 1;

const STACKTRACE_ENTRY_REGEX: &Lazy<Regex> = regex!(r"^\s+((at)|(\.{3})) ");
const STACKTRACE_ENTRY_START: &str = "                                         ";

const STACKTRACE_CAUSE_REGEX: &Lazy<Regex> = regex!(r"^\s*Caused by: ");
const STACKTRACE_CAUSE_START: &str = "                                   ";

pub struct LogcatFormatter {
    tag_length: Cell<usize>,
}

impl LogcatFormatter {
    pub const fn new() -> Self {
        Self {
            tag_length: Cell::new(0),
        }
    }

    pub fn format_log_line(&self, log_line: &LogLine) -> FormattedLogLine {
        match log_line {
            LogLine::Header(header) => Self::format_log_header(header),
            LogLine::Entry(entry) => self.format_log_entry(entry),
        }
    }

    fn format_log_header(header: &str) -> FormattedLogLine {
        format!("--------- beginning of {header}")
            .color(colors::HEADER)
            .to_log_line()
    }

    fn format_log_entry(&self, entry: &LogEntry) -> FormattedLogLine {
        if Self::is_stacktrace_entry(entry) {
            Self::format_stacktrace_entry(entry)
        } else if Self::is_stacktrace_cause(entry) {
            Self::format_stacktrace_cause(entry)
        } else {
            self.format_regular_entry(entry)
        }
    }

    fn is_stacktrace_entry(entry: &LogEntry) -> bool {
        STACKTRACE_ENTRY_REGEX.is_match(&entry.message)
    }

    fn is_stacktrace_cause(entry: &LogEntry) -> bool {
        STACKTRACE_CAUSE_REGEX.is_match(&entry.message)
    }

    fn format_stacktrace_entry(entry: &LogEntry) -> FormattedLogLine {
        let tag_padding = " ".repeat(entry.tag.len());
        format!("{STACKTRACE_ENTRY_START}{tag_padding}{}", entry.message.trim_start())
            .color(Self::color_of_level(entry.level))
            .dimmed()
            .to_log_line()
    }

    fn format_stacktrace_cause(entry: &LogEntry) -> FormattedLogLine {
        let tag_padding = " ".repeat(entry.tag.len());
        format!("{STACKTRACE_CAUSE_START}{tag_padding}{}", entry.message.trim_start())
            .color(Self::color_of_level(entry.level))
            .to_log_line()
    }

    fn format_regular_entry(&self, entry: &LogEntry) -> FormattedLogLine {
        let parts = nev![
            Self::format_datetime(&entry.datetime),
            Self::format_pid(entry.pid),
            Self::format_tid(entry.tid),
            Self::format_log_level(entry.level),
            self.format_tag(&entry.tag),
            Self::format_message(&entry.message, entry.level),
        ];
        FormattedLogLine::new(parts)
    }

    fn format_datetime(datetime: &UtcDateTime) -> ColoredString {
        datetime
            .format(DATETIME_FORMAT)
            .unwrap_or_else(|e| panic!("failed to format datetime: {e}"))
            .color(colors::DATETIME)
    }

    fn format_pid(pid: u32) -> ColoredString {
        format!("{pid:>5}").color(colors::PID)
    }

    fn format_tid(tid: u32) -> ColoredString {
        format!("{tid:>5}").color(colors::TID)
    }

    fn format_log_level(level: LogLevel) -> ColoredString {
        format!(" {level} ")
            .color(colors::levels::FOREGROUND)
            .on_color(Self::color_of_level(level))
            .bold()
    }

    fn format_tag(&self, tag: &str) -> ColoredString {
        if tag.len() > self.tag_length.get() {
            self.tag_length.set(tag.len().min(MAX_TAG_LENGTH));
        }

        let padded_tag = format!("{tag:^width$}", width = self.tag_length.get());

        match tag {
            MISSING_TAG => padded_tag.color(colors::MISSING_TAG).italic(),
            _ => padded_tag.color(colors::TAG),
        }
    }

    fn format_message(message: &str, level: LogLevel) -> ColoredString {
        message.color(Self::color_of_level(level))
    }

    const fn color_of_level(level: LogLevel) -> Color {
        match level {
            LogLevel::Silent => colors::levels::SILENT,
            LogLevel::Verbose => colors::levels::VERBOSE,
            LogLevel::Debug => colors::levels::DEBUG,
            LogLevel::Info => colors::levels::INFO,
            LogLevel::Warning => colors::levels::WARNING,
            LogLevel::Error => colors::levels::ERROR,
            LogLevel::Fatal => colors::levels::FATAL,
        }
    }
}

#[extend::ext]
impl ColoredString {
    fn to_log_line(self) -> FormattedLogLine {
        FormattedLogLine::new(nev![self])
    }
}
