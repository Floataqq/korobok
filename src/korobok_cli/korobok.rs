mod parser;
mod runner;

use anyhow::Result;
use clap::Parser;
use parser::{Command, KorobokOptions};
use runner::run;

fn main() -> Result<()> {
    let args = KorobokOptions::try_parse()?;

    match args.command {
        Command::Run(rd) => run(rd, args.global_opts)?,
    }

    Ok(())
}
