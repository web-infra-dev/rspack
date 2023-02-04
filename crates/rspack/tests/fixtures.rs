use std::path::PathBuf;

use rspack_test::read_test_config_and_normalize;
use rspack_test_utils::test_fixture_rst;
use rspack_tracing::enable_tracing_by_env;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture_rst(&fixture_path);
}

#[fixture("tests/samples/**/test.config.json")]
fn samples(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture_rst(fixture_path.parent().expect("should exist"));
}

#[tokio::main]
async fn run(context: PathBuf) {
  let mut options = read_test_config_and_normalize(&context);
  options.__emit_error = true;
  let mut compiler = rspack::rspack(options, vec![]);
  compiler.run().await.expect("TODO:");
}

#[fixture("../../examples/*")]
fn example(fixture_path: PathBuf) {
  run(fixture_path);
}

#[fixture("tests/tree-shaking/*", exclude("node_modules"))]
fn tree_shaking(fixture_path: PathBuf) {
  test_fixture_rst(&fixture_path);
}
