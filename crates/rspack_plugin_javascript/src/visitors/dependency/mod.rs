mod code_generation;
mod common_js_scanner;
mod harmony_detection;
mod hmr_scanner;
mod import_meta_scanner;
mod node_stuff_scanner;
mod scanner;
mod util;
pub use code_generation::*;
use rspack_core::{
  ast::javascript::Program, BuildInfo, BuildMeta, CompilerOptions, Dependency, ModuleDependency,
  ModuleType, ResourceData,
};
use swc_core::common::{comments::Comments, Mark, SyntaxContext};
pub use util::*;

use self::{
  common_js_scanner::CommonJsScanner, harmony_detection::HarmonyDetectionScanner,
  hmr_scanner::HmrDependencyScanner, import_meta_scanner::ImportMetaScanner,
  node_stuff_scanner::NodeStuffScanner, scanner::DependencyScanner,
};

pub type ScanDependenciesResult = (Vec<Box<dyn ModuleDependency>>, Vec<Box<dyn Dependency>>);

pub fn scan_dependencies(
  program: &Program,
  unresolved_mark: Mark,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  build_info: &mut BuildInfo,
  build_meta: &mut BuildMeta,
) -> ScanDependenciesResult {
  let mut dependencies: Vec<Box<dyn ModuleDependency>> = vec![];
  let mut presentational_dependencies: Vec<Box<dyn Dependency>> = vec![];
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  // Comments is wrapped by Arc/Rc
  let comments = program.comments.clone();
  program.visit_with_path(
    &mut DependencyScanner::new(
      &unresolved_ctxt,
      resource_data,
      compiler_options,
      &mut dependencies,
      &mut presentational_dependencies,
      comments.as_ref().map(|c| c as &dyn Comments),
    ),
    &mut Default::default(),
  );
  program.visit_with_path(
    &mut HmrDependencyScanner::new(&mut dependencies),
    &mut Default::default(),
  );

  if module_type.is_js_auto() || module_type.is_js_dynamic() {
    program.visit_with_path(
      &mut CommonJsScanner::new(&mut dependencies, &mut presentational_dependencies),
      &mut Default::default(),
    );

    if let Some(node_option) = &compiler_options.node {
      program.visit_with_path(
        &mut NodeStuffScanner::new(
          &mut presentational_dependencies,
          &unresolved_ctxt,
          compiler_options,
          node_option,
          resource_data,
        ),
        &mut Default::default(),
      );
    }
  }

  if module_type.is_js_auto() || module_type.is_js_esm() {
    program.visit_with_path(
      &mut ImportMetaScanner::new(
        &mut presentational_dependencies,
        resource_data,
        compiler_options,
      ),
      &mut Default::default(),
    );
    program.visit_with(&mut HarmonyDetectionScanner::new(
      build_info,
      build_meta,
      module_type,
      &mut presentational_dependencies,
    ));
  }

  (dependencies, presentational_dependencies)
}
