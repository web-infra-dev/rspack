use temp_test_utils::read_test_config_and_normalize;
#[tokio::main]
async fn main() {
  let mut cur_dir = std::env::current_dir().unwrap();
  cur_dir = cur_dir.join("tests/fixtures/webpack/at-charset");
  println!("{:?}", cur_dir);
  let options = read_test_config_and_normalize(&cur_dir);

  println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .run()
    .await
    .unwrap_or_else(|e| panic!("{:?}, failed to compile in fixtrue {:?}", e, cur_dir));
  // println!("{:?}", stats);
}
