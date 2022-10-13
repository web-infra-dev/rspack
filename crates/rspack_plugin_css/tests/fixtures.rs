use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;
#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
#[fixture("tests/fixtures/postcss/*")]
fn postcss(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[fixture("tests/fixtures/custom/*")]
fn custom(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[test]
#[inline(never)]
#[doc(hidden)]
fn webpack_css_tests__fixtures__webpack__multiple_entry_2() {
  eprintln!("Input: {}","/Users/bytedance/Projects/rspack/crates/rspack_plugin_css/tests/fixtures/webpack/multiple-entry");
  webpack_css(::std::path::PathBuf::from("/Users/bytedance/Projects/rspack/crates/rspack_plugin_css/tests/fixtures/webpack/multiple-entry"));
}

#[test]
#[inline(never)]
#[doc(hidden)]
fn webpack_css_tests__fixtures__webpack__shared_import_2() {
  eprintln!("Input: {}","/Users/bytedance/Projects/rspack/crates/rspack_plugin_css/tests/fixtures/webpack/shared-import");
  webpack_css(::std::path::PathBuf::from("/Users/bytedance/Projects/rspack/crates/rspack_plugin_css/tests/fixtures/webpack/shared-import"));
}
