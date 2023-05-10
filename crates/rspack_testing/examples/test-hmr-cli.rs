//! cargo run --example test-cli -- `pwd`/tests/fixtures/simple

use std::{env, path::PathBuf};

use rspack_testing::test_rebuild_fixture;

fn main() {
  let fixture = PathBuf::from("crates/rspack_testing/examples/simple");
  let fixture = if fixture.is_absolute() {
    fixture
  } else {
    let cwd = env::current_dir().expect("current_dir");
    cwd.join(fixture).canonicalize().expect("canonicalize")
  };
  test_rebuild_fixture(
    &fixture,
    Some(Box::new(|_compiler| {
      // dbg!(compiler.compilation.include_module_ids);
    })),
  );
}
