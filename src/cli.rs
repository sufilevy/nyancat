use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct Args {
    /// Read and process input from stdin [default].
    #[arg(long, conflicts_with_all(["input_file", "exec_adb_logcat"]), default_value_t = true)]
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

    /// Execute `adb logcat`, capture and process its output.
    #[arg(long("exec-logcat"), default_value_t = false)]
    exec_adb_logcat: bool,
}
