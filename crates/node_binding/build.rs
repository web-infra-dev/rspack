fn main() {
  // Check for mutually exclusive features
  let sftrace_setup = std::env::var("CARGO_FEATURE_SFTRACE_SETUP").is_ok();
  let allocative = std::env::var("CARGO_FEATURE_ALLOCATIVE").is_ok();

  if sftrace_setup && allocative {
    eprintln!(
      "error: Features 'sftrace-setup' and 'allocative' are mutually exclusive and cannot be enabled together"
    );
    std::process::exit(1);
  }

  rspack_binding_build::setup();

  #[cfg(feature = "sftrace-setup")]
  {
    sftrace_setup();
  }
}

#[cfg(feature = "sftrace-setup")]
fn sftrace_setup() {
  use std::path::PathBuf;

  fn search_sftracelib() -> Option<PathBuf> {
    use std::process::{Command, Stdio};

    let result = Command::new("sftrace")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .arg("record")
      .arg("--print-solib")
      .output();

    match result {
      Ok(output) if output.status.success() => {
        let out = String::from_utf8(output.stdout).ok()?;
        let mut out = PathBuf::from(out);
        out.pop();
        Some(out)
      }
      _ => None,
    }
  }

  if let Some(lib) = search_sftracelib() {
    println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib.display());
  } else {
    println!("cargo::warning=not found sftrace");
  }
}
