use std::path::PathBuf;

use rspack_testing::{fixture, test_fixture};

#[fixture("tests/fixtures/**/*.config.js*")]
fn wasm(config_path: PathBuf) {
  let fixture_path = config_path.parent().expect("TODO:");
  test_fixture(fixture_path);
}
