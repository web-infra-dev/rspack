use std::path::PathBuf;
use temp_test_utils::test_fixture;
use testing_macros::fixture;

#[fixture("fixtures/webpack/*")]
fn webpack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[fixture("fixtures/rspack/*")]
fn rspack_asset(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
