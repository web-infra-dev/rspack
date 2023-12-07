use std::path::PathBuf;
use std::sync::atomic::Ordering;

use cargo_rst::git_diff;
use rspack_core::{
  BoxPlugin, CompilerOptions, MangleExportsOption, PluginExt, TreeShaking, UsedExportsOption,
  IS_NEW_TREESHAKING,
};
use rspack_plugin_javascript::{
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, MangleExportsPlugin,
  SideEffectsFlagPlugin,
};
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
    Box::new(
      |plugins: &mut Vec<BoxPlugin>, options: &mut CompilerOptions| {
        options.experiments.rspack_future.new_treeshaking = true;
        options.optimization.provided_exports = true;
        options.optimization.used_exports = UsedExportsOption::Global;
        plugins.push(Box::<FlagDependencyExportsPlugin>::default());
        plugins.push(Box::<FlagDependencyUsagePlugin>::default());
        if options.optimization.mangle_exports.is_enable() {
          plugins.push(
            MangleExportsPlugin::new(!matches!(
              options.optimization.mangle_exports,
              MangleExportsOption::Size
            ))
            .boxed(),
          );
        }
      },
    ),
    None,
  );
}

#[fixture("tests/tree-shaking/*", exclude("node_modules"))]
fn tree_shaking(fixture_path: PathBuf) {
  // For each test case
  // First test is old version tree shaking snapshot test
  test_fixture(&fixture_path, Box::new(|_, _| {}), None);
  // second test is webpack based tree shaking
  IS_NEW_TREESHAKING.store(true, Ordering::SeqCst);
  test_fixture(
    &fixture_path,
    Box::new(
      |plugins: &mut Vec<BoxPlugin>, options: &mut CompilerOptions| {
        options.experiments.rspack_future.new_treeshaking = true;
        options.optimization.provided_exports = true;
        options.optimization.inner_graph = true;
        options.optimization.used_exports = UsedExportsOption::True;
        options.builtins.tree_shaking = TreeShaking::False;

        if options.optimization.side_effects.is_enable() {
          plugins.push(Box::<SideEffectsFlagPlugin>::default());
        }
        plugins.push(Box::<FlagDependencyExportsPlugin>::default());
        plugins.push(Box::<FlagDependencyUsagePlugin>::default());
      },
    ),
    Some("new_treeshaking".to_string()),
  );

  // then we generate a diff file, the less diff generated the more we are closed to our
  // target
  let old_snapshot_path = fixture_path.join("snapshot/output.snap");
  let old_snapshot = std::fs::read_to_string(old_snapshot_path).expect("should have snapshot");
  let new_treeshaking_snapshot_path = fixture_path.join("snapshot/new_treeshaking.snap");
  let new_treeshaking_snapshot =
    std::fs::read_to_string(new_treeshaking_snapshot_path).expect("should have snapshot");
  let diff = git_diff(&old_snapshot, &new_treeshaking_snapshot);
  let diff_path = fixture_path.join("snapshot/snap.diff");
  if diff_path.exists() {
    std::fs::remove_file(diff_path.clone()).expect("remove file failed");
  }
  if !diff.is_empty() {
    std::fs::write(diff_path, diff).expect("should write successfully");
  }
}
