use std::{env, process::Command};

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-env-changed=RUSTC");

  // src/arc.rs relies on private alloc::sync layouts. Never silently carry that
  // implementation across a toolchain update, even if its size checks pass.
  let rustc = env::var_os("RUSTC").expect("Cargo must provide RUSTC");
  let output = Command::new(rustc)
    .arg("-vV")
    .output()
    .expect("failed to inspect rustc for the temporary Arc conversion");
  assert!(output.status.success(), "rustc -vV failed");
  let version = String::from_utf8(output.stdout).expect("rustc version must be UTF-8");
  assert!(
    version
      .lines()
      .any(|line| line == "commit-hash: e8e4541ff19649d95afab52fdde2c2eaa6829965"),
    "rspack_util::arc requires the audited nightly-2026-04-16 standard library; \
     replace it with Arc::try_into_unique when available, or audit the new \
     alloc::sync layouts and conversion before updating this check"
  );
}
