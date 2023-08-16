use clap;
use clap::ArgMatches;

use super::stats_diff_command;
use crate::runner::RunnerOptions;

#[derive(Debug)]
pub struct DistDiffRunnerOptions {
  pub src_path: String,
  pub dst_path: String,
}
impl RunnerOptions for DistDiffRunnerOptions {
  fn build_args(cmd: clap::Command) -> clap::Command {
    stats_diff_command(cmd)
  }
}
impl From<&ArgMatches> for DistDiffRunnerOptions {
  fn from(matches: &ArgMatches) -> Self {
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
