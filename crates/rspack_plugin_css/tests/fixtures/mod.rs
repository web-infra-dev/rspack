use crate::common::test_fixture_css;
use std::path::PathBuf;
use testing_macros::fixture;

#[fixture("fixtures/*")]
fn css(fixture_path: PathBuf) {
  // let is_ignored = fixture_path
  //   .file_name()
  //   .map(|s| s.to_string_lossy().to_string())
  //   .map(|path| path.starts_with('_'))
  //   .unwrap_or(true);

  // if !is_ignored {
  tokio::runtime::Runtime::new()
    .unwrap()
    .block_on(test_fixture_css(&fixture_path));
  // }
}
