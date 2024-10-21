use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::utils::command::run_command;

#[derive(Debug, Parser)]
pub struct ApiExtractorCmd {
  #[clap(subcommand)]
  sub_cmd: ApiExtractorSubCmd,
}

#[derive(Debug, Subcommand)]
pub enum ApiExtractorSubCmd {
  Update,
  Ci,
}

impl ApiExtractorCmd {
  pub fn run(&self) -> Result<()> {
    match &self.sub_cmd {
      ApiExtractorSubCmd::Update => {
        run_command("pnpm", &["-w", "build:js"])?;
        run_command(
          "pnpm",
          &["--filter", "@rspack/*", "api-extractor", "--local"],
        )?;
      }
      ApiExtractorSubCmd::Ci => {
        run_command("pnpm", &["--filter", "@rspack/*", "api-extractor:ci"]).with_context(|| {
          "Api-extractor testing failed. Did you forget to update the snapshots locally?\nRun the command below locally to fix this error (in the *ROOT* of rspack workspace).\n$ ./x api-extractor update"
        })?;
      }
    }
    Ok(())
  }
}
