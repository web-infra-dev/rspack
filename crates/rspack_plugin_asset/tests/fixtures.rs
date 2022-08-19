use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;
#[fixture("fixtures/webpack/*")]
fn webpack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[fixture("fixtures/rspack/*")]
fn rspack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
