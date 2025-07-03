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
