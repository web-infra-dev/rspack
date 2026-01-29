use signal_hook::{consts::signal::SIGUSR2, iterator::Signals};
pub fn should_stop() -> bool {
  std::env::var("RSPACK_DEBUG_STOP").is_ok()
}
// wait for user input to continue, useful for debugging and profiling purposes
// inspired by https://github.com/rust-lang/rust-analyzer/blob/661e7d2ac245f4fca099d982544b3c5408322867/crates/rust-analyzer/src/bin/main.rs#L97
pub fn wait_for_enter(phase: &str) {
  if !should_stop() {
    return;
  }
  println!(
    "Waiting in phase {} with pid={}, Press Enter to continue.",
    phase,
    std::process::id()
  );
  let mut s = String::new();
  let _ = std::io::stdin().read_line(&mut s);
}
// wait for signal to continue
pub fn wait_for_signal(phase: &str) {
  if !should_stop() {
    return;
  }
  println!(
    "Waiting in phase {}, run `kill -SIGUSR2 {}` to continue.",
    phase,
    std::process::id()
  );
  let mut signal = Signals::new([SIGUSR2]).expect("Failed to create signal handler");
  for sig in signal.forever() {
    println!("Received signal: {:?}", sig);
    break;
  }
}
