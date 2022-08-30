use std::path::PathBuf;

use rspack_test::read_test_config_and_normalize;
use tokio::runtime::Builder;
// #[tokio::main]
fn main() {
  let mut vec = vec![];
  for _ in 0..2 {
    let handle = std::thread::spawn(|| {
      let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();
      runtime.block_on(async { test().await });
    });
    vec.push(handle);
  }

  for handle in vec {
    handle.join().unwrap();
  }
  // let manifest_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  // let bundle_dir = manifest_dir.join("tests/fixtures/webpack/at-charset");
  // // manifest_dir = man
  // println!("{:?}", manifest_dir);
  // let options = read_test_config_and_normalize(&bundle_dir);

  // // println!("{:?}", options);
  // let mut compiler = rspack::rspack(options, Default::default());

  // let _stats = compiler
  //   .run()
  //   .await
  //   .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, bundle_dir));
  // println!("{:?}", _stats);
}

async fn test() {
  let manifest_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let bundle_dir = manifest_dir.join("tests/fixtures/webpack/at-charset");
  // manifest_dir = manifest_dir.join("../../examples/bench");
  println!("{:?}", manifest_dir);
  let options = read_test_config_and_normalize(&bundle_dir);

  // println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .build()
    .await
    .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, bundle_dir));
}
