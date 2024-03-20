use std::path::PathBuf;

use rspack_core::{
  BoxPlugin, CompilerOptions, MangleExportsOption, PluginExt, TreeShaking, UsedExportsOption,
};
use rspack_plugin_javascript::{
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, MangleExportsPlugin,
  ModuleConcatenationPlugin, SideEffectsFlagPlugin,
};
use rspack_testing::test_fixture;
use testing_macros::fixture;

#[fixture("tests/fixtures/*")]
fn rspack(fixture_path: PathBuf) {
  test_fixture(&fixture_path, Box::new(|_, _| {}), None);
}

fn is_used_exports_global(options: &CompilerOptions) -> bool {
  matches!(options.optimization.used_exports, UsedExportsOption::Global)
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
        plugins.push(FlagDependencyUsagePlugin::new(is_used_exports_global(options)).boxed());
        if options.optimization.side_effects.is_enable() {
          plugins.push(Box::<SideEffectsFlagPlugin>::default());
        }
        if options.optimization.mangle_exports.is_enable() {
          plugins.push(
            MangleExportsPlugin::new(!matches!(
              options.optimization.mangle_exports,
              MangleExportsOption::Size
            ))
            .boxed(),
          );
        }
        if options.optimization.concatenate_modules {
          plugins.push(Box::<ModuleConcatenationPlugin>::default());
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
  // test_fixture(
  //   &fixture_path,
  //   Box::new(
  //     |plugins: &mut Vec<BoxPlugin>, options: &mut CompilerOptions| {
  //       options.experiments.rspack_future.new_treeshaking = true;
  //       options.optimization.provided_exports = true;
  //       options.optimization.inner_graph = true;
  //       options.optimization.used_exports = UsedExportsOption::True;
  //       options.builtins.tree_shaking = TreeShaking::False;

  //       if options.optimization.side_effects.is_enable() {
  //         plugins.push(Box::<SideEffectsFlagPlugin>::default());
  //       }
  //       if options.optimization.mangle_exports.is_enable() {
  //         plugins.push(
  //           MangleExportsPlugin::new(!matches!(
  //             options.optimization.mangle_exports,
  //             MangleExportsOption::Size
  //           ))
  //           .boxed(),
  //         );
  //       }
  //       plugins.push(Box::<FlagDependencyExportsPlugin>::default());
  //       plugins.push(FlagDependencyUsagePlugin::new(is_used_exports_global(options)).boxed());
  //     },
  //   ),
  //   Some("new_treeshaking".to_string()),
  // );
}
