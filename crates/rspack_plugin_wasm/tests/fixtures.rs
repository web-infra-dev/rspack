use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture};

#[fixture("tests/fixtures/*")]
fn wasm(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
