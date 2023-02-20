//! cargo run --example build-cli -- `pwd`/tests/fixtures/simple .

use std::{env, path::PathBuf};

use rspack_testing::test_fixture;

fn main() {
  let fixture = env::args().nth(1).expect("path");
  let fixture = PathBuf::from(fixture);
  let fixture = if fixture.is_absolute() {
    fixture
  } else {
    let cwd = env::current_dir().expect("current_dir");
    cwd.join(fixture).canonicalize().expect("canonicalize")
  };
  test_fixture(&fixture);
}
