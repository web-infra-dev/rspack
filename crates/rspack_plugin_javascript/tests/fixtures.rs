use std::path::PathBuf;

use rspack_core::CompilerOptions;
use rspack_testing::{add_entry_runtime, fixture, test_fixture_with_modify};

#[fixture("tests/fixtures/**/*.config.json")]
fn js(config_path: PathBuf) {
  let fixture_path = config_path.parent().expect("TODO:");
  test_fixture_with_modify(fixture_path, |options: CompilerOptions| {
    add_entry_runtime(options)
  });
}
