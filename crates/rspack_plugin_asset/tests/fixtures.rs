use rspack_core::CompilerOptions;
use rspack_test::{add_entry_runtime, fixture, test_fixture};
use std::path::PathBuf;
#[fixture("fixtures/webpack/*")]
fn webpack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}

#[fixture("fixtures/rspack/*")]
fn rspack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}
