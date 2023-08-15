use clap::ArgMatches;

use crate::{
  result::CliRunResult,
  runner::{Runner, RunnerOptions},
};

#[derive(Debug)]
pub struct DistDiffRunnerOptions {
  pub src_path: String,
  pub dst_path: String,
}

impl RunnerOptions for DistDiffRunnerOptions {
  fn build_args(cmd: clap::Command) -> clap::Command {
    cmd
  }
}
impl From<ArgMatches> for DistDiffRunnerOptions {
  fn from(matches: ArgMatches) -> Self {
    Self {
      src_path: matches
        .get_one::<String>("src_path")
        .expect("msg_path is required")
        .to_owned(),
      dst_path: matches
        .get_one::<String>("dst_path")
        .expect("dst_path is requried")
        .to_owned(),
    }
  }
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
    todo!()
  }
  fn run(&self) -> CliRunResult {
    todo!()
  }
}
