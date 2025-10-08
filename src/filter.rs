use anyhow::anyhow;
use regex::{Regex, RegexBuilder};

use crate::{
    log::{LogEntry, LogLevel},
    prelude::*,
};

pub trait LogFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool;
}

pub type BoxedLogFilter = Box<dyn LogFilter>;

pub struct AndFilter(pub Vec<BoxedLogFilter>);

impl LogFilter for AndFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        self.0.iter().all(|filter| filter.include_entry(log_entry))
    }
}

pub struct OrFilter(pub Vec<BoxedLogFilter>);

impl LogFilter for OrFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        self.0.iter().any(|filter| filter.include_entry(log_entry))
    }
}

pub struct NotFilter(pub BoxedLogFilter);

impl LogFilter for NotFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        !self.0.include_entry(log_entry)
    }
}

pub struct PidFilter(pub u32);

impl LogFilter for PidFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        log_entry.pid == self.0
    }
}

pub struct TidFilter(pub u32);

impl LogFilter for TidFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        log_entry.tid == self.0
    }
}

pub struct LevelFilter(pub LogLevel);

impl LogFilter for LevelFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        log_entry.level >= self.0
    }
}

pub struct TagFilter(Regex);

impl TagFilter {
    pub fn new(regex: &str) -> Result<Self> {
        let regex = RegexBuilder::new(&format!("^{regex}$"))
            .build()
            .map_err(|e| anyhow!("failed to compile regex: {e}"))?;
        Ok(Self(regex))
    }
}

impl LogFilter for TagFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        self.0.is_match(&log_entry.tag)
    }
}

pub struct MessageFilter(pub Regex);

impl LogFilter for MessageFilter {
    fn include_entry(&self, log_entry: &LogEntry) -> bool {
        self.0.is_match(&log_entry.message)
    }
}
