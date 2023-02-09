use std::path::PathBuf;

use rspack_core::CompilerOptions;
use rspack_testing::{add_entry_runtime, fixture, test_fixture_with_modify};

#[fixture("tests/fixtures/webpack/*")]
fn webpack_asset(fixture_path: PathBuf) {
  test_fixture_with_modify(&fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}

#[fixture("tests/fixtures/rspack/*")]
fn rspack_asset(fixture_path: PathBuf) {
  test_fixture_with_modify(&fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}
