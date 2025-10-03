use strum::Display;
use time::UtcDateTime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LogLine {
    Header(String),
    Entry(LogEntry),
}

impl LogLine {
    pub fn header(header: &str) -> Self {
        Self::Header(header.to_owned())
    }

    pub const fn entry(
        datetime: UtcDateTime,
        pid: u32,
        tid: u32,
        level: LogLevel,
        tag: String,
        message: String,
    ) -> Self {
        Self::Entry(LogEntry::new(datetime, pid, tid, level, tag, message))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogEntry {
    pub datetime: UtcDateTime,
    pub pid: u32,
    pub tid: u32,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
}

impl LogEntry {
    pub const fn new(datetime: UtcDateTime, pid: u32, tid: u32, level: LogLevel, tag: String, message: String) -> Self {
        Self {
            datetime,
            pid,
            tid,
            level,
            tag,
            message,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Display, strum::EnumString, strum::EnumIter)]
pub enum LogLevel {
    #[strum(serialize = "S")]
    Silent,
    #[strum(serialize = "V")]
    Verbose,
    #[strum(serialize = "D")]
    Debug,
    #[strum(serialize = "I")]
    Info,
    #[strum(serialize = "W")]
    Warning,
    #[strum(serialize = "E")]
    Error,
    #[strum(serialize = "F")]
    Fatal,
}
