use std::{path::PathBuf, time::Instant};

use rspack_test::read_test_config_and_normalize;
#[tokio::main]
async fn main() {
  let manifest_dir = PathBuf::from(&std::env::var("PWD").unwrap());
  // let bundle_dir = manifest_dir.join("tests/fixtures/postcss/pxtorem");
  let bundle_dir: PathBuf = manifest_dir.join("benchcases/three");
  println!("{:?}", bundle_dir);
  let mut options = read_test_config_and_normalize(&bundle_dir);

  options.emit_error = true;
  let start = Instant::now();
  // println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, bundle_dir));
  println!("{:?}", start.elapsed());
}
