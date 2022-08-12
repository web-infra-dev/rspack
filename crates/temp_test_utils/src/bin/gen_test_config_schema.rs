use std::path::PathBuf;

use schemars::schema_for;
use temp_test_utils::TestOptions;

fn main() {
  let schema = schema_for!(TestOptions);
  let scheme_path =
    PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("test.config.scheme.json");
  std::fs::write(scheme_path, &serde_json::to_string_pretty(&schema).unwrap()).unwrap();
}
