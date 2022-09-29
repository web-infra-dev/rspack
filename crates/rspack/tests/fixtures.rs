use std::path::PathBuf;

use rspack_core::log::enable_tracing_by_env;
use rspack_test::{read_test_config_and_normalize, test_fixture};
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

// #[tokio::main]
// #[test]
// async fn test_lodash() {
//   let start = std::time::SystemTime::now();
//   let path = std::path::Path::new("/Users/bytedance/rspack/benchcases/lodash-with-simple-css");
//   let options = read_test_config_and_normalize(&path);
//   let mut compiler = rspack::rspack(options, vec![]);
//   compiler.run().await.unwrap();
//   println!("cost: {:?}", start.elapsed());
// }
