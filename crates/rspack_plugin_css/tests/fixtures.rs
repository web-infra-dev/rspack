use hashbrown::HashMap;
use rspack::Compiler;
use rspack_node::{normalize_bundle_options, RawOptions};
use std::path::{Path, PathBuf};
use rspack_test::fixture;

#[fixture("tests/fixtures/webpack/*")]
fn webpack_css(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
