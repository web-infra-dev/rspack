use std::path::PathBuf;
use temp_test_utils::test_fixture;
use testing_macros::fixture;

#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[fixture("tests/fixtures/postcss/*")]
fn postcss(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
