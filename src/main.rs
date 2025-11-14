#![allow(dead_code)]

use crate::{cli::run_cli, prelude::*};

mod cli;
mod filter;
mod format;
mod input;
mod log;
mod parse;
mod prelude;

fn main() -> Result<()> {
    run_cli()?;
    Ok(())
}
