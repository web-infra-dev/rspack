use std::{env, ffi::OsString};

use clap::{self, Args, Parser, Subcommand};
pub mod helper;
mod record;
pub mod rst;
mod terminal_inline;
mod update;
pub use rst::test;
use update::update;

#[derive(Parser)]
#[clap(version, name = "cargo_rst", about, long_about = None)]
struct Cli {
  #[clap(subcommand)]
  commands: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  Update {
    #[clap(short, long, value_parser)]
    path: Option<String>,
  },

  Test(TestCmd),
}

#[derive(Args, Debug)]
struct TestCmd {
  /// Whether output file diff details.
  #[clap(short, long, action, default_value_t = false)]
  detail: bool,

  /// If fail, no output, default to false.
  #[clap(short, long, action, default_value_t = false)]
  mute: bool,

  /// If true, record will not be saved on the disk.
  #[clap(short, long, action, default_value_t = true)]
  no_write: bool,

  /// Options passed to cargo test
  // Sets raw to true so that `--` is required
  #[clap(name = "cargo_options", raw(true))]
  cargo_options: Vec<String>,
}

#[allow(clippy::unwrap_used)]
pub fn setup(args: &Vec<OsString>) {
  let cli = Cli::parse_from(args);

  match &cli.commands.expect("TODO:") {
    Commands::Update { path } => {
      update(path.clone());
    }
    Commands::Test(cmd) => {
      if cmd.detail {
        env::set_var("RST_DETAIL", "1");
      }

      if cmd.mute {
        env::set_var("RST_MUTE", "1");
      }

      let mut proc = std::process::Command::new(
        env::var("CARGO")
          .ok()
          .unwrap_or_else(|| "cargo".to_string()),
      );
      proc.arg("test");

      proc.args(&cmd.cargo_options);
      proc.arg("--");
      proc.arg("-q");

      proc.status().expect("TODO:");
    }
  };
}
