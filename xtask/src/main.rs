use clap::{Parser, Subcommand};

use crate::{codegen::CodegenCmd, deny_ext::DenyExtCmd, release_check::ReleaseCheckCmd};
mod codegen;
mod deny_ext;
mod release_check;

#[derive(Debug, Parser)]
struct CliArgs {
  #[clap(subcommand)]
  cmd: Cmd,
}
#[derive(Debug, Subcommand)]
enum Cmd {
  /// cargo deny extension to enforce more rule about dependency management
  DenyExt(DenyExtCmd),
  /// codegenerate for workspace version
  Codegen(CodegenCmd),
  /// check release criteria for all crates in the workspace
  ReleaseCheck(ReleaseCheckCmd),
}
fn main() -> anyhow::Result<()> {
  let args = CliArgs::parse();
  match args.cmd {
    Cmd::DenyExt(c) => c.run()?,
    Cmd::Codegen(c) => c.run()?,
    Cmd::ReleaseCheck(c) => c.run()?,
  }
  Ok(())
}
