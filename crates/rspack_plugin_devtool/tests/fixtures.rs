use rspack_test::{fixture, rspack_only::options_noop, test_fixture};
use std::path::PathBuf;

#[fixture("tests/fixtures/**/*.config.js")]
fn source_map(config_path: PathBuf) {
  let fixture_path = config_path.parent().expect("TODO:");
  test_fixture(fixture_path, options_noop);
}
