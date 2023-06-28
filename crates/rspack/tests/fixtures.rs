use std::path::PathBuf;

use rspack_testing::test_fixture;
use rspack_tracing::enable_tracing_by_env;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path);
}

#[fixture("tests/fixtures/code-splitting")]
fn rspack2(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path);
}

#[fixture("tests/samples/**/test.config.json")]
fn samples(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(fixture_path.parent().expect("should exist"));
}

#[fixture("tests/tree-shaking/*", exclude("node_modules"))]
fn tree_shaking(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
