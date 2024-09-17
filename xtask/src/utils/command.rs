use std::process::Command;

use anyhow::{Context, Result};

#[cfg(windows)]
const PNPM: &str = "pnpm.cmd";
#[cfg(not(windows))]
const PNPM: &str = "pnpm";

pub fn run_command(command: &str, args: &[&str]) -> Result<()> {
  let program = match command {
    "pnpm" => PNPM,
    other => other,
  };

  let status = Command::new(program)
    .args(args)
    .status()
    .context("Failed to execute command")?;

  if status.success() {
    Ok(())
  } else {
    Err(anyhow::anyhow!(
      "Command `{}` failed with status code: {}",
      command,
      status.code().unwrap_or(-1)
    ))
  }
}
