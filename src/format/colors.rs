use colored::Color;

pub const HEADER: Color = Color::BrightBlack;

pub const DATETIME: Color = Color::Magenta;
pub const PID: Color = Color::BrightBlack;
pub const TID: Color = Color::BrightBlack;
pub const TAG: Color = Color::BrightWhite;

pub mod levels {
    use super::*;

    pub const FOREGROUND: Color = Color::Black;

    pub const VERBOSE: Color = Color::Cyan;
    pub const DEBUG: Color = Color::Blue;
    pub const INFO: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
}
