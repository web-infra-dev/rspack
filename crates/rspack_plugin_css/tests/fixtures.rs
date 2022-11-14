use rspack_test::{fixture, rspack_only::options_noop, test_fixture};
use std::path::PathBuf;
#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path, options_noop);
}
#[fixture("tests/fixtures/postcss/*")]
fn postcss(fixture_path: PathBuf) {
  test_fixture(&fixture_path, options_noop);
}

#[fixture("tests/fixtures/custom/*")]
fn custom(fixture_path: PathBuf) {
  test_fixture(&fixture_path, options_noop);
}
