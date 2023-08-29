use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture_insta};

#[fixture("tests/fixtures/*")]
fn html(fixture_path: PathBuf) {
  test_fixture_insta(&fixture_path, &|filename: &str| -> bool {
    filename.contains(".html")
  });
}
