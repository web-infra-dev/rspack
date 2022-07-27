use std::path::PathBuf;
use temp_test_utils::test_fixture;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn js(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
