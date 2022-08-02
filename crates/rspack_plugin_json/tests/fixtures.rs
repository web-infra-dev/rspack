use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;

#[fixture("tests/fixtures/*")]
fn json(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
