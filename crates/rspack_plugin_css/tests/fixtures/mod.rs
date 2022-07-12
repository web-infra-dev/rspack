use crate::common::test_fixture_css;
use std::path::PathBuf;
use testing_macros::fixture;

#[fixture("fixtures/*")]
fn css(fixture_path: PathBuf) {
  tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(test_fixture_css(&fixture_path));
}
