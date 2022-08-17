use std::path::PathBuf;
use rspack_test::{ test_fixture , fixture};
#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
