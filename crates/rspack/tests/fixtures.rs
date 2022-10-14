use std::path::PathBuf;

use rspack_test::{read_test_config_and_normalize, test_fixture};
use rspack_tracing::enable_tracing_by_env;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  enable_tracing_by_env();
  test_fixture(&fixture_path);
}

#[tokio::main]
async fn run(context: PathBuf) {
  let options = read_test_config_and_normalize(&context);
  let mut compiler = rspack::rspack(options, vec![]);
  compiler.run().await.unwrap();
}

#[fixture("../../examples/*")]
fn example(fixture_path: PathBuf) {
  run(fixture_path);
}

#[test]
#[inline(never)]
#[doc(hidden)]
fn example______examples__react_with_less2() {
  eprintln!(
    "Input: {}",
    "/Users/bytedance/Projects/rspack/examples/react-with-less"
  );
  example(::std::path::PathBuf::from(
    "/Users/bytedance/Projects/rspack/examples/react-with-less",
  ));
}
