use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture_css};

#[fixture("tests/fixtures/*")]
fn sass(fixture_path: PathBuf) {
  test_fixture_css(&fixture_path);
}
