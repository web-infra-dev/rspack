use std::path::{Path, PathBuf};

use crate::common::compile_fixture;

use sugar_path::PathSugar;

#[tokio::test]
async fn plugin_globals() {
  compile_fixture("plugin-globals").await;
  let string = std::fs::read_to_string(Path::resolve(&PathBuf::from("./dist/main.js"))).unwrap();
  insta::assert_snapshot!(string);
}
