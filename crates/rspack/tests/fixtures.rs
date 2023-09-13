use std::path::PathBuf;

use insta::Settings;
use rspack_core::{CompilerOptions, TreeShaking, UsedExportsOption};
use rspack_testing::test_fixture;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  test_fixture(&fixture_path, Box::new(|_, _| {}), None);
}

#[fixture("tests/samples/**/test.config.json")]
fn samples(fixture_path: PathBuf) {
  test_fixture(
    fixture_path.parent().expect("should exist"),
    Box::new(|_, _| {}),
    None,
  );
}

#[fixture("tests/tree-shaking/*", exclude("node_modules"))]
fn tree_shaking(fixture_path: PathBuf) {
  // For each test case
  // First test is old version tree shaking snapshot test
  test_fixture(&fixture_path, Box::new(|_, _| {}), None);
  // second test is webpack based tree shaking
  test_fixture(
    &fixture_path,
    Box::new(|settings: &mut Settings, options: &mut CompilerOptions| {
      options.experiments.rspack_future.new_treeshaking = true;
      options.optimization.provided_exports = true;
      options.optimization.used_exports = UsedExportsOption::True;
      options.builtins.tree_shaking = TreeShaking::False;
    }),
    Some("new_treeshaking".to_string()),
  );
  // then we generate a diff file, the less diff generated it means the more we are closed to our
  // target
}
