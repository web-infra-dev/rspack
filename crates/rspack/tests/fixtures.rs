use std::path::PathBuf;

use insta::Settings;
use rspack_binding_options::RawOptions;
use rspack_core::tree_shaking::visitor::TreeShakingResult;
use rspack_core::Compilation;
use rspack_core::CompilerOptions;
use rspack_symbol::Symbol;
use rspack_test::{read_test_config_and_normalize, test_fixture, test_options::RawOptionsExt};
use rspack_tracing::enable_tracing_by_env;
use testing_macros::fixture;
use ustr::Ustr;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path, |mut options: CompilerOptions| {
    options.__wrap_runtime = false;
    options
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

#[fixture("tests/tree-shaking/*")]
fn tree_shaking(fixture_path: PathBuf) {
  test_fixture(&fixture_path, |mut options: CompilerOptions| {
    options.__wrap_runtime = false;
    options
  });
}
