#![feature(internal_output_capture)]
use std::{path::PathBuf, sync::Arc};

use rspack_test::read_test_config_and_normalize;
#[tokio::main]
async fn main() {
  let local = Default::default();
  std::io::set_output_capture(Some(local));
  let manifest_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let bundle_dir = manifest_dir.join("tests/fixtures/webpack/at-charset");
  // manifest_dir = manifest_dir.join("../../examples/bench");
  println!("{:?}", manifest_dir);
  let options = read_test_config_and_normalize(&bundle_dir);

  // println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .run()
    .await
    .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, bundle_dir));
  // _stats.compilation.
  let captured = std::io::set_output_capture(None);
  let captured = captured.unwrap();
  let captured = Arc::try_unwrap(captured).unwrap();
  let captured = captured.into_inner().unwrap();
  let captured = String::from_utf8(captured).unwrap();

  // println!("{}", captured);
  // println!("{:?}", _stats);
}
