use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture};

#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[fixture("tests/fixtures/custom/*")]
fn custom(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
