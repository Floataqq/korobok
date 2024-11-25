mod parser;
mod runner;

use anyhow::Result;
use clap::Parser;
use parser::{Command, KorobokOptions};

fn main() -> Result<()> {
    let args = KorobokOptions::try_parse()?;

    match args.command {
        Command::Run(rd) => println!("{}", runner::run(rd, args.global_opts)?),
    }

    Ok(())
}
