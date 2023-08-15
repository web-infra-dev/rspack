use rspack_diff::{
  command::command,
  dist_diff::{DistDiffRunner, DistDiffRunnerOptions},
  runner::Runner,
};

fn main() {
  let matches = command().get_matches();
  match matches.subcommand() {
    Some((subcommand, options)) => match subcommand {
      DistDiffRunner::NAME => {
        let options = DistDiffRunnerOptions::from(matches);
        DistDiffRunner::new(options).run();
      }
      _ => {
        unreachable!("not supported custom command {subcommand}")
      }
    },
    _ => unreachable!("required pass subcommand"),
  }
}
