use rspack_test::{fixture, rspack_only::options_noop, test_fixture};
use std::path::PathBuf;

#[fixture("fixtures/*")]
fn html(fixture_path: PathBuf) {
  test_fixture(&fixture_path, options_noop);
}
