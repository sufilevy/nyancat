#![allow(dead_code)]

use crate::prelude::*;

mod cli;
mod filter;
mod format;
mod input;
mod log;
mod parse;
mod prelude;

fn main() -> Result<()> {
    cli::run()
}
