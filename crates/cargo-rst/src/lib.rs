// use std::{env, ffi::OsString};

pub mod helper;
mod record;
pub mod rst;
mod terminal_inline;
mod update;
pub use rst::test;
pub use update::update;

// #[derive(FromArgs)]
// struct Cli {
//   #[argh(subcommand)]
//   commands: Commands,
// }
//
// #[derive(FromArgs, Debug)]
// #[argh(subcommand)]
// enum Commands {
//   Update {
//     #[argh(short, long, value_parser)]
//     path: Option<String>,
//   },
//
//   Test(TestCmd),
// }
//
// #[derive(Args, Debug)]
// struct TestCmd {
//   /// Whether output file diff details.
//   #[argh(short, default_value = "default_false()")]
//   detail: bool,
//
//   /// If fail, no output, default to false.
//   #[argh(short, long, action, default_value_t = "default_false()")]
//   mute: bool,
//
//   /// If true, record will not be saved on the disk.
//   #[argh(short, long, action, default_value_t = "default_true()")]
//   no_write: bool,
//
//   /// Options passed to cargo test
//   // Sets raw to true so that `--` is required
//   #[argh(name = "cargo_options", raw(true))]
//   cargo_options: Vec<String>,
// }
//
// fn default_false() -> bool {
//   false
// }
//
// fn default_true() -> bool {
//   false
// }

// #[allow(clippy::unwrap_used)]
// pub fn setup(args: &Vec<OsString>) {
//   let cli: Cli = argh::from_env();
//
//   match cli.commands {
//     Commands::Update { path } => {
//       update(path.clone());
//     }
//     Commands::Test(cmd) => {
//       if cmd.detail {
//         env::set_var("RST_DETAIL", "1");
//       }
//
//       if cmd.mute {
//         env::set_var("RST_MUTE", "1");
//       }
//
//       let mut proc = std::process::Command::new(
//         env::var("CARGO")
//           .ok()
//           .unwrap_or_else(|| "cargo".to_string()),
//       );
//       proc.arg("test");
//
//       proc.args(&cmd.cargo_options);
//       proc.arg("--");
//       proc.arg("-q");
//
//       proc.status().expect("TODO:");
//     }
//   };
// }
