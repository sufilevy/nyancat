use std::cell::Cell;

use colored::{Color, ColoredString, Colorize};
use lazy_regex::{Lazy, regex};
use nonempty_collections::nev;
use regex::Regex;
use time::{UtcDateTime, format_description::BorrowedFormatItem};
use time_macros::format_description;

use super::{colors, log_line::FormattedLogLine};
use crate::log::{LogEntry, LogLevel, LogLine};

const DATETIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");

const MAX_TAG_LENGTH: usize = 1;

const STACKTRACE_ENTRY_REGEX: &Lazy<Regex> = regex!(r"^\s+at ");
const STACKTRACE_ENTRY_START: &str = "                                         ";

const STACKTRACE_CAUSE_REGEX: &Lazy<Regex> = regex!(r"^\s*Caused by: ");
const STACKTRACE_CAUSE_START: &str = "                                   ";

pub struct LogcatFormatter {
    tag_length: Cell<usize>,
}

impl LogcatFormatter {
    pub fn new() -> Self {
        Self {
            tag_length: Cell::new(0),
        }
    }

    pub fn format_log_line(&self, log_line: &LogLine) -> FormattedLogLine {
        match log_line {
            LogLine::Header(header) => self.format_log_header(header),
            LogLine::Entry(entry) => self.format_log_entry(entry),
        }
    }

    fn format_log_header(&self, header: &str) -> FormattedLogLine {
        format!("--------- beginning of {}", header)
            .color(colors::HEADER)
            .to_log_line()
    }

    fn format_log_entry(&self, entry: &LogEntry) -> FormattedLogLine {
        if self.is_stacktrace_entry(entry) {
            self.format_stacktrace_entry(entry)
        } else if self.is_stacktrace_cause(entry) {
            self.format_stacktrace_cause(entry)
        } else {
            self.format_regular_entry(entry)
        }
    }

    fn is_stacktrace_entry(&self, entry: &LogEntry) -> bool {
        STACKTRACE_ENTRY_REGEX.is_match(&entry.message)
    }

    fn is_stacktrace_cause(&self, entry: &LogEntry) -> bool {
        STACKTRACE_CAUSE_REGEX.is_match(&entry.message)
    }

    fn format_stacktrace_entry(&self, entry: &LogEntry) -> FormattedLogLine {
        let tag_padding = " ".repeat(entry.tag.len());
        format!("{STACKTRACE_ENTRY_START}{tag_padding}{}", entry.message.trim_start())
            .color(self.color_of_level(entry.level))
            .dimmed()
            .to_log_line()
    }

    fn format_stacktrace_cause(&self, entry: &LogEntry) -> FormattedLogLine {
        let tag_padding = " ".repeat(entry.tag.len());
        format!("{STACKTRACE_CAUSE_START}{tag_padding}{}", entry.message.trim_start())
            .color(self.color_of_level(entry.level))
            .to_log_line()
    }

    fn format_regular_entry(&self, entry: &LogEntry) -> FormattedLogLine {
        let parts = nev![
            self.format_datetime(&entry.datetime),
            self.format_pid(entry.pid),
            self.format_tid(entry.tid),
            self.format_log_level(entry.level),
            self.format_tag(&entry.tag),
            self.format_message(&entry.message, entry.level),
        ];
        FormattedLogLine::new(parts)
    }

    fn format_datetime(&self, datetime: &UtcDateTime) -> ColoredString {
        datetime
            .format(DATETIME_FORMAT)
            .unwrap_or_else(|e| panic!("failed to format datetime: {e}"))
            .color(colors::DATETIME)
    }

    fn format_pid(&self, pid: u32) -> ColoredString {
        format!("{pid:>5}").color(colors::PID)
    }

    fn format_tid(&self, tid: u32) -> ColoredString {
        format!("{tid:>5}").color(colors::TID)
    }

    fn format_log_level(&self, level: LogLevel) -> ColoredString {
        format!(" {} ", level)
            .color(colors::levels::FOREGROUND)
            .on_color(self.color_of_level(level))
            .bold()
            .dimmed()
    }

    fn format_tag(&self, tag: &str) -> ColoredString {
        if tag.len() > self.tag_length.get() {
            self.tag_length.set(tag.len().min(MAX_TAG_LENGTH));
        }
        format!("{tag:^width$}", width = self.tag_length.get())
            .color(colors::TAG)
            .bold()
    }

    fn format_message(&self, message: &str, level: LogLevel) -> ColoredString {
        message.color(self.color_of_level(level))
    }

    fn color_of_level(&self, level: LogLevel) -> Color {
        match level {
            LogLevel::Verbose => colors::levels::VERBOSE,
            LogLevel::Debug => colors::levels::DEBUG,
            LogLevel::Info => colors::levels::INFO,
            LogLevel::Warning => colors::levels::WARNING,
            LogLevel::Error => colors::levels::ERROR,
        }
    }
}

#[extend::ext]
impl ColoredString {
    fn to_log_line(self) -> FormattedLogLine {
        FormattedLogLine::new(nev![self])
    }
}
