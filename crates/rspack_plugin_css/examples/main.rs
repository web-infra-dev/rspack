use std::{collections::HashMap, os::linux::raw::stat, path::Path};

use node_binding::{normalize_bundle_options, RawOptions};
use rspack_core::{AssetContent, Compiler};
use temp_test_utils::RawOptionsTestExt;
#[tokio::main]
async fn main() {
  let mut cur_dir = std::env::current_dir().unwrap();
  println!("{:?}", cur_dir);
  cur_dir = cur_dir.join("cases_to_be_migrated/at-charset");
  let options = normalize_bundle_options(RawOptions::from_fixture(&cur_dir))
    .unwrap_or_else(|_| panic!("failed to normalize in fixtrue {:?}", cur_dir));
  println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let stats = compiler
    .run()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", cur_dir));
  println!("{:?}", stats);
}
