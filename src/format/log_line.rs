use colored::ColoredString;
use nonempty_collections::NEVec;

pub struct FormattedLogLine {
    parts: NEVec<ColoredString>,
}

impl FormattedLogLine {
    pub const fn new(parts: NEVec<ColoredString>) -> Self {
        Self { parts }
    }
}

impl std::fmt::Display for FormattedLogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parts.first())?;
        for part in self.parts.iter().skip(1) {
            write!(f, " {part}")?;
        }
        Ok(())
    }
}
