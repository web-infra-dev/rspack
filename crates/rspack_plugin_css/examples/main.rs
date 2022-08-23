use rspack_test::read_test_config_and_normalize;
#[tokio::main]
async fn main() {
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
  // println!("{:?}", _stats);
}
