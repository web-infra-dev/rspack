use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;

#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
