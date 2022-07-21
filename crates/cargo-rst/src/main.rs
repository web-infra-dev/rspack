use std::env;

use cargo_rst::setup;

fn main() {
  let mut args: Vec<_> = env::args_os().collect();

  if env::var("CARGO").is_ok() && args.get(1).and_then(|x| x.to_str()) == Some("rst") {
    args.remove(1);
  }

  setup(&args);
}
