fn main() {
  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");

  // Check for mutually exclusive features
  let sftrace_setup = std::env::var("CARGO_FEATURE_SFTRACE_SETUP").is_ok();
  let allocative = std::env::var("CARGO_FEATURE_ALLOCATIVE").is_ok();

  if sftrace_setup && allocative {
    eprintln!(
      "error: Features 'sftrace-setup' and 'allocative' are mutually exclusive and cannot be enabled together"
    );
    std::process::exit(1);
  }
}
