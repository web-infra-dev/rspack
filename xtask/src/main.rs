use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::*;

mod commands;
mod utils;

#[derive(Debug, Parser)]
#[clap(
  name = "Rspack Development CLI",
  version = "1.0",
  about = "CLI for development of Rspack"
)]
struct CliArgs {
  #[clap(subcommand)]
  cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
  #[clap(name = "api-extractor")]
  ApiExtractor(api_extractor::ApiExtractorCmd),
}

fn main() -> Result<()> {
  let args = CliArgs::parse();

  match args.cmd {
    Cmd::ApiExtractor(cmd) => cmd.run(),
  }
}
