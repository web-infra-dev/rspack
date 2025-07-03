// wait for user input to continue, useful for debugging and profiling purposes
// inspired by https://github.com/rust-lang/rust-analyzer/blob/661e7d2ac245f4fca099d982544b3c5408322867/crates/rust-analyzer/src/bin/main.rs#L97
#[cfg(feature = "debug_tool")]
pub fn wait_for(phase: &str) {
  println!(
    "Waiting in phase {}, for debugger | samply to attach (pid {})... Press Enter to continue.",
    phase,
    std::process::id()
  );
  let mut s = String::new();
  let _ = std::io::stdin().read_line(&mut s);
}
