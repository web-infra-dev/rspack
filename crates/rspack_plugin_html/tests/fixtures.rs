use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;

#[fixture("fixtures/*")]
fn html(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
