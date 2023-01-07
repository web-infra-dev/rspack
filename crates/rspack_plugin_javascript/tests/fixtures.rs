use std::path::PathBuf;

use rspack_core::CompilerOptions;
use rspack_test::{add_entry_runtime, fixture, test_fixture};

#[fixture("tests/fixtures/**/*.config.js")]
fn js(config_path: PathBuf) {
  let fixture_path = config_path.parent().expect("TODO:");
  test_fixture(fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}
