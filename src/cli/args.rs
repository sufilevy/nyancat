use clap::{Parser, ValueHint};

use crate::log::LogLevel;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct Args {
    /// Read input from stdin [default when input is piped into the program].
    #[arg(long, conflicts_with_all(["input_file", "exec_adb_logcat"]), default_value_t = false)]
    pub stdin: bool,

    #[clap(flatten)]
    pub input: Input,

    #[clap(flatten)]
    pub filter: Filter,
}

#[derive(Debug, Parser)]
#[group(required(false), multiple(false))]
pub struct Input {
    /// Path to an input file to read and process.
    #[arg(long("file"), value_name("FILE"), value_hint(ValueHint::FilePath))]
    pub input_file: Option<String>,

    /// Execute `adb logcat`, capture its output and process it [default when no input is piped into the program].
    #[arg(long("exec-logcat"), default_value_t = false)]
    pub exec_adb_logcat: bool,
}

#[derive(Debug, Parser)]
#[group(required(false), multiple(true))]
pub struct Filter {
    /// Only include log lines logged from a process with this pid.
    #[arg(long)]
    pub pid: Option<u32>,

    /// Only include log lines logged from a thread with this tid.
    #[arg(long)]
    pub tid: Option<u32>,

    /// Only include log lines with this level or higher.
    #[arg(short('L'), long, value_name("V|D|I|W|E"))]
    pub level: Option<LogLevel>,

    /// Only include log lines with one of the specified tags (see more with '--help')
    ///
    /// Each specified tag is matched as a regex against the tag of each line, and if one of the tags matches, the line
    /// is included.
    ///
    /// The passed regex is wrapped in `^<regex>$`, meaning the regex must match the entire tag.
    /// For example:
    /// - Passing `Tag` will only match `Tag`.
    /// - Passing `Tag\d` will match `Tag1`, `Tag2`, etc.
    /// - Passing `Tag.*` will match anything starting with `Tag`, e.g. `Tag`, `TagABC`, `Tag123`...
    #[arg(
        short('T'),
        long,
        value_delimiter(','),
        value_name("TAG[,TAG...]"),
        verbatim_doc_comment
    )]
    pub tag: Option<Vec<String>>,

    /// Only include log lines with a message matching the specified regex (see more with '--help')
    ///
    /// Can be repeated by passing this argument multiple times.
    ///
    /// Each specified regex is matched against the message of each line, and if one of them matches, the line is
    /// included.
    ///
    /// The passed regex is used as-is.
    /// For example:
    /// - Passing `Word` will match any message containing `Word`.
    /// - Passing `Word\d` will match any message containing `Word1`, `Word2`, etc.
    /// - Passing `^Word` will match any message starting with `Word`.
    #[arg(short('M'), long, value_name("REGEX"), verbatim_doc_comment)]
    pub message: Option<Vec<String>>,
}
