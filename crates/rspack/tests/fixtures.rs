use std::path::PathBuf;

use rspack_test::{read_test_config_and_normalize, test_fixture};
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[tokio::main]
async fn run(context: PathBuf) {
  let options = read_test_config_and_normalize(&context);
  let mut compiler = rspack::rspack(options, vec![]);
  compiler.compile().await.unwrap();
}

#[fixture("../../examples/react")]
fn example(fixture_path: PathBuf) {
  run(fixture_path);
}
