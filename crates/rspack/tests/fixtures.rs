use std::path::PathBuf;

use rspack_core::CompilerOptions;
use rspack_test::{add_entry_runtime, read_test_config_and_normalize, test_fixture};
use rspack_tracing::enable_tracing_by_env;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}

#[tokio::main]
async fn run(context: PathBuf) {
  let mut options = read_test_config_and_normalize(&context);
  options.__emit_error = true;
  let mut compiler = rspack::rspack(options, vec![]);
  compiler.run().await.unwrap();
}

#[fixture("../../examples/*")]
fn example(fixture_path: PathBuf) {
  run(fixture_path);
}

#[fixture("tests/tree-shaking/*", exclude("node_modules"))]
fn tree_shaking(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}
