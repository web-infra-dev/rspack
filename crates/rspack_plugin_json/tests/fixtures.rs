use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture};

#[fixture("tests/fixtures/*")]
fn json(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
