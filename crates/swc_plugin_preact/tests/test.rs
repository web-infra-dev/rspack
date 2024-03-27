use std::path::PathBuf;

use swc_core::{
  common::{chain, Mark},
  ecma::transforms::{base::resolver, testing::test_fixture},
  testing,
};
use swc_plugin_preact::{plugin_preact, PluginPreactConfig};

#[testing::fixture("tests/fixtures/**/input.js")]
fn fixture(input: PathBuf) {
  let output = input
    .parent()
    .expect("should have parent directory")
    .join("output.js");

  test_fixture(
    Default::default(),
    &|_| {
      chain!(
        resolver(Mark::new(), Mark::new(), false),
        plugin_preact(
          PluginPreactConfig {
            library: Some("@custom/preact".to_string()),
          },
          "__file_hash__".to_string(),
        )
      )
    },
    &input,
    &output,
    Default::default(),
  );
}
