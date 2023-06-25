mod api_scanner;
mod common_js_import_dependency_scanner;
mod common_js_scanner;
mod harmony_detection_scanner;
mod harmony_export_dependency_scanner;
mod harmony_import_dependency_scanner;
mod hot_module_replacement_scanner;
mod import_meta_scanner;
mod import_scanner;
mod node_stuff_scanner;
mod require_context_scanner;
mod scanner;
mod url_scanner;
mod util;
use rspack_core::{
  ast::javascript::Program, BuildInfo, BuildMeta, BuildMetaExportsType, CodeGeneratableDependency,
  CompilerOptions, ModuleDependency, ModuleIdentifier, ModuleType, ResourceData,
};
use swc_core::common::{comments::Comments, Mark, SyntaxContext};
pub use util::*;

use self::{
  api_scanner::ApiScanner, common_js_import_dependency_scanner::CommonJsImportDependencyScanner,
  common_js_scanner::CommonJsScanner, harmony_detection_scanner::HarmonyDetectionScanner,
  harmony_export_dependency_scanner::HarmonyExportDependencyScanner,
  harmony_import_dependency_scanner::HarmonyImportDependencyScanner,
  hot_module_replacement_scanner::HotModuleReplacementScanner,
  import_meta_scanner::ImportMetaScanner, import_scanner::ImportScanner,
  node_stuff_scanner::NodeStuffScanner, require_context_scanner::RequireContextScanner,
  url_scanner::UrlScanner,
};

pub type ScanDependenciesResult = (
  Vec<Box<dyn ModuleDependency>>,
  Vec<Box<dyn CodeGeneratableDependency>>,
);

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  program: &Program,
  unresolved_mark: Mark,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  build_info: &mut BuildInfo,
  build_meta: &mut BuildMeta,
  module_identifier: ModuleIdentifier,
) -> ScanDependenciesResult {
  let mut dependencies: Vec<Box<dyn ModuleDependency>> = vec![];
  let mut presentational_dependencies: Vec<Box<dyn CodeGeneratableDependency>> = vec![];
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  let comments = program.comments.clone();

  program.visit_with(&mut ApiScanner::new(
    &unresolved_ctxt,
    resource_data,
    &mut presentational_dependencies,
  ));

  // TODO it should enable at js/auto or js/dynamic, but builtins provider will inject require at esm
  program.visit_with(&mut CommonJsImportDependencyScanner::new(
    &mut dependencies,
    &mut presentational_dependencies,
    &unresolved_ctxt,
  ));
  if module_type.is_js_auto() || module_type.is_js_dynamic() {
    // TODO webpack scan it at CommonJsExportsParserPlugin
    // use `Dynamic` as workaround
    build_meta.exports_type = BuildMetaExportsType::Dynamic;
    program.visit_with(&mut CommonJsScanner::new(&mut presentational_dependencies));
    program.visit_with(&mut RequireContextScanner::new(&mut dependencies));

    if let Some(node_option) = &compiler_options.node {
      program.visit_with(&mut NodeStuffScanner::new(
        &mut presentational_dependencies,
        &unresolved_ctxt,
        compiler_options,
        node_option,
        resource_data,
      ));
    }
  }

  program.visit_with(&mut ImportScanner::new(
    &mut dependencies,
    comments.as_ref().map(|c| c as &dyn Comments),
  ));

  if module_type.is_js_auto() || module_type.is_js_esm() {
    program.visit_with(&mut HarmonyDetectionScanner::new(
      build_info,
      build_meta,
      module_type,
      &mut presentational_dependencies,
    ));
    let mut import_map = Default::default();
    program.visit_with(&mut HarmonyImportDependencyScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      &mut import_map,
      module_identifier,
    ));
    program.visit_with(&mut HarmonyExportDependencyScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      &mut import_map,
      module_identifier,
    ));
    program.visit_with(&mut UrlScanner::new(&mut dependencies));
    program.visit_with(&mut ImportMetaScanner::new(
      &mut presentational_dependencies,
      resource_data,
      compiler_options,
    ));
  }

  if compiler_options.dev_server.hot {
    program.visit_with(&mut HotModuleReplacementScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      module_identifier,
      build_meta,
    ));
  }

  (dependencies, presentational_dependencies)
}
