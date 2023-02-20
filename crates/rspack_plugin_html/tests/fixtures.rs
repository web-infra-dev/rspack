use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture};

#[fixture("tests/fixtures/*")]
fn html(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
