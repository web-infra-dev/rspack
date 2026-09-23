/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

fn main() {
  println!("cargo:rustc-check-cfg=cfg(rust_nightly)");
  println!("cargo:rustc-check-cfg=cfg(rust_unstable_lazy_get)");
  rust_nightly();
}

fn rust_nightly() {
  let rustc = std::env::var("RUSTC").expect("Cargo sets RUSTC for build scripts");
  let version = std::process::Command::new(rustc)
    .arg("--version")
    .output()
    .expect("rustc must report its version");

  assert!(version.status.success());

  // Nightly output:
  // rustc 1.64.0-nightly (affe0d3a0 2022-08-05)
  // Stable output:
  // rustc 1.64.0 (a55dd71d5 2022-09-19)
  // Meta internal output:
  // rustc 1.64.0-dev

  let stdout = String::from_utf8(version.stdout).expect("rustc version output is UTF-8");
  assert!(stdout.contains("rustc"), "Sanity check");
  let nightly = stdout.contains("nightly") || stdout.contains("dev");
  if nightly {
    println!("cargo:rustc-cfg=rust_nightly");
    // Dylint uses an older nightly where LazyLock::get still needs this gate.
    // Do not enable the gate on newer Rust: stable_features is denied in CI.
    let minor = stdout
      .split_whitespace()
      .nth(1)
      .and_then(|version| version.split('.').nth(1))
      .and_then(|minor| minor.parse::<u32>().ok())
      .expect("rustc version contains a minor version");
    if minor < 94 {
      println!("cargo:rustc-cfg=rust_unstable_lazy_get");
    }
  }
}
