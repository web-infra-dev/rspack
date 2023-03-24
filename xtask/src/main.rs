mod copy_three;
use std::env::args;

#[allow(clippy::dbg_macro)]
fn main() {
  let args = args().skip(1).collect::<Vec<_>>();
  // dbg!(&args);
  let command = &args[0];
  match command.as_ref() {
    "copy_three" => {
      let num = args
        .get(1)
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or(10);
      copy_three::copy_three(num);
    }
    "three_production_config" => {
      copy_three::three_production_config();
    }
    _ => (),
  }
}
