use clap::{Arg, Command};

pub fn dist_diff_command() -> Command {
  Command::new("dist_diff")
    .about("diff the bundled output of webpack and rspack")
    .arg_required_else_help(true)
    .arg(Arg::new("src_path"))
    .arg(Arg::new("dst_path"))
}
pub fn stats_diff_command() -> Command {
  Command::new("stats_diff")
    .about("diff the bundled stats of webpack and rspack")
    .arg_required_else_help(true)
    .arg(Arg::new("rspack_path"))
}

pub fn command() -> Command {
  Command::new("rspack_diff")
    .bin_name("rspack_diff")
    .about("diff webpack and rspack build result")
    .arg_required_else_help(true)
    .subcommand_required(true)
    .subcommand(dist_diff_command())
    .subcommand(stats_diff_command())
}
