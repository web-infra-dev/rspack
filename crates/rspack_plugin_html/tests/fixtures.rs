use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture_html};

#[fixture("tests/fixtures/*")]
fn html(fixture_path: PathBuf) {
  test_fixture_html(&fixture_path);
}
