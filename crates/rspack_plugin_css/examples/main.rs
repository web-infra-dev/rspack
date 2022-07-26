use node_binding::{normalize_bundle_options, RawOptions};
use rspack_test::rspack_only::RawOptionsTestExt;
#[tokio::main]
async fn main() {
  let mut cur_dir = std::env::current_dir().unwrap();
  cur_dir = cur_dir.join("webpack_css_cases_to_be_migrated/at-import-in-the-middle");
  println!("{:?}", cur_dir);
  let options = normalize_bundle_options(RawOptions::from_fixture(&cur_dir))
    .unwrap_or_else(|_| panic!("failed to normalize in fixtrue {:?}", cur_dir));
  println!("{:?}", options);
  let mut compiler = rspack::rspack(options, Default::default());

  let _stats = compiler
    .run()
    .await
    .unwrap_or_else(|_| panic!("failed to compile in fixtrue {:?}", cur_dir));
  // println!("{:?}", stats);
}
