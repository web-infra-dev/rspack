use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture};

#[fixture("tests/fixtures/webpack/*")]
fn webpack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[fixture("tests/fixtures/rspack/*")]
fn rspack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
