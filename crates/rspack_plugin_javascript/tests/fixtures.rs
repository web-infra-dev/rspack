use rspack_core::CompilerOptions;
use rspack_test::{add_entry_runtime, fixture, test_fixture};
use std::path::PathBuf;

#[fixture("tests/fixtures/**/*.config.js")]
fn js(config_path: PathBuf) {
  let fixture_path = config_path.parent().unwrap();
  test_fixture(fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}
