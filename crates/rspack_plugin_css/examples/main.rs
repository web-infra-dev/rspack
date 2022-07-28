use std::path::{Path, PathBuf};

use node_binding::{normalize_bundle_options, RawOptions};
use temp_test_utils::RawOptionsTestExt;
#[tokio::main]
async fn main() {
  let mut cur_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  cur_dir = cur_dir.join("../../examples/bench");
  cur_dir = cur_dir.canonicalize().unwrap();
  println!("{:?}", cur_dir);
  // cur_dir = cur_dir.join("webpack_css_cases_to_be_migrated/bootstrap");
  let options = normalize_bundle_options(RawOptions::from_fixture(&cur_dir))
    .unwrap_or_else(|_| panic!("failed to normalize in fixtrue {:?}", cur_dir));
  println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let stats = compiler
    .run()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", cur_dir));
  // println!("{:?}", stats);
}
