mod copy_three;
use std::env::args;

fn main() {
  let args = args().into_iter().skip(1).collect::<Vec<_>>();
  dbg!(&args);
  let command = &args[0];
  if let "copy_three" = command.as_ref() {
    let num = args
      .get(1)
      .and_then(|num| num.parse::<usize>().ok())
      .unwrap_or(10);
    copy_three::copy_three(num);
  }
}
