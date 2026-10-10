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
  println!("cargo:rerun-if-env-changed=RSPACK_ALLOCATIVE_DRIVER");
  if std::env::var_os("CARGO_CFG_ALLOCATIVE").is_some() {
    assert!(
      std::env::var("RSPACK_ALLOCATIVE_DRIVER").as_deref() == Ok("1"),
      "allocative builds require the compiler driver: pnpm exec zx scripts/allocative-driver.mjs cargo <arguments>"
    );
  }
}
