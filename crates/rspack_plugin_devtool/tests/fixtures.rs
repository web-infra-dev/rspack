use rspack_test::{fixture, test_fixture};
use std::path::PathBuf;

#[fixture("tests/fixtures/**/*.config.js")]
fn source_map(config_path: PathBuf) {
  let fixture_path = config_path.parent().unwrap();
  test_fixture(fixture_path);
}
