use clap::{builder::ValueParser, Arg, Command};

use crate::{result::CliRunResult, runner::Runner};

#[derive(Debug)]
pub struct DistDiffRunnerOptions {
  pub src_path: String,
  pub dst_path: String,
}

#[derive(Debug)]
pub struct DistDiffRunner {
  options: DistDiffRunnerOptions,
}
#[derive(Debug)]
pub struct DistDiffResult {}

impl Runner for DistDiffRunner {
  const NAME: &'static str = "dist_diff";
  const ABOUT: &'static str = "diff the bundle dist";
  type Options = DistDiffRunnerOptions;

  fn new(options: Self::Options) -> Self {
    dbg!(options);
    todo!()
  }
  fn run(&self) -> CliRunResult {
    todo!()
  }
}

pub fn stats_diff_command(command: Command) -> Command {
  command
    .arg_required_else_help(true)
    .arg(Arg::new("src_path"))
    .arg(Arg::new("dst_path"))
}
