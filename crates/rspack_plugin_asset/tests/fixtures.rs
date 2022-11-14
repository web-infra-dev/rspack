use rspack_test::{fixture, rspack_only::options_noop, test_fixture};
use std::path::PathBuf;
#[fixture("fixtures/webpack/*")]
fn webpack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path, options_noop);
}

#[fixture("fixtures/rspack/*")]
fn rspack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path, options_noop);
}
