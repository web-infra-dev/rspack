use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;

#[fixture("fixtures/*")]
fn js(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
