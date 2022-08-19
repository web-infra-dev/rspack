use std::fs;
use std::path::PathBuf;

use rspack_binding_options::{normalize_bundle_options, RawOptions};
use temp_test_utils::test_fixture;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}

#[tokio::main]
async fn run(context: PathBuf) {
  let config_path = context
    .join("test.config.json")
    .to_string_lossy()
    .to_string();
  let config = fs::read_to_string(config_path).unwrap();
  let options: RawOptions = serde_json::from_str(&config).expect("load config failed");
  let mut compiler = rspack::rspack(
    normalize_bundle_options(RawOptions {
      context: Some(context.to_string_lossy().to_string()),
      ..options
    })
    .unwrap(),
    vec![],
  );
  compiler.compile().await.unwrap();
}

#[fixture("../../examples/react")]
fn react(fixture_path: PathBuf) {
  run(fixture_path);
}
