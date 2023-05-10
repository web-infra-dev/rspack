//! cargo run --example test-cli -- `pwd`/tests/fixtures/simple

use std::{env, path::PathBuf};

use rspack_testing::test_hmr_fixture;

fn main() {
  let fixture = PathBuf::from("crates/rspack_testing/examples/simple");
  dbg!(&fixture);
  let fixture = if fixture.is_absolute() {
    fixture
  } else {
    let cwd = env::current_dir().expect("current_dir");
    dbg!(&cwd);
    cwd.join(fixture).canonicalize().expect("canonicalize")
  };
  test_hmr_fixture(&fixture);
}
