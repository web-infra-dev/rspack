use clap;
use clap::ArgMatches;
use clap::Command;

use super::stats_diff_command;
use super::DistDiffRunnerOptions;
use crate::runner::RunnerOptions;

impl RunnerOptions for DistDiffRunnerOptions {
  fn build_args(cmd: clap::Command) -> clap::Command {
    stats_diff_command(cmd)
  }
}

impl From<ArgMatches> for DistDiffRunnerOptions {
  fn from(matches: ArgMatches) -> Self {
    dbg!(&matches);
    let x = matches.get_one::<String>("dst_path");
    dbg!(x);
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
