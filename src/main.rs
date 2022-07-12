use crate::client::Clients;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;

use self::{client::Client, parse::parse, process_line::ProcessLine};

mod client;
mod parse;
mod process_line;
mod to_csv;

#[derive(Parser)]
struct Cli {
    filename: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut clients = Clients::default();

    for tx_rec in parse(cli.filename)? {
        tx_rec
            .process_line(&mut clients)
            .unwrap_or_else(|e| eprintln!("{}", e));
    }

    clients
        .to_csv()
        .or_else(|e| Err(anyhow!("Error serializing data: {}", e)))?;

    Ok(())
}
