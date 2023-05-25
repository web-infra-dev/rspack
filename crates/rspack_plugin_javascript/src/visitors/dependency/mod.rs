mod api_scanner;
mod code_generation;
mod common_js_import_dependency_scanner;
mod common_js_scanner;
mod harmony_detection_scanner;
mod harmony_export_dependency_scanner;
mod harmony_import_dependency_scanner;
mod hmr_scanner;
mod import_meta_scanner;
mod import_scanner;
mod node_stuff_scanner;
mod require_context_scanner;
mod scanner;
mod util;
pub use code_generation::*;
use rspack_core::{
  ast::javascript::Program, BuildInfo, BuildMeta, CodeReplaceSourceDependency, CompilerOptions,
  Dependency, ModuleDependency, ModuleType, ResourceData,
};
use swc_core::common::{comments::Comments, Mark, SyntaxContext};
pub use util::*;

use self::{
  api_scanner::ApiScanner, common_js_import_dependency_scanner::CommonJsImportDependencyScanner,
  common_js_scanner::CommonJsScanner, harmony_detection_scanner::HarmonyDetectionScanner,
  harmony_export_dependency_scanner::HarmonyExportDependencyScanner,
  harmony_import_dependency_scanner::HarmonyImportDependencyScanner,
  hmr_scanner::HmrDependencyScanner, import_meta_scanner::ImportMetaScanner,
  import_scanner::ImportScanner, node_stuff_scanner::NodeStuffScanner,
  require_context_scanner::RequireContextScanner, scanner::DependencyScanner,
};

pub type ScanDependenciesResult = (
  Vec<Box<dyn ModuleDependency>>,
  Vec<Box<dyn Dependency>>,
  Vec<Box<dyn CodeReplaceSourceDependency>>,
);

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
    program.visit_with(&mut HarmonyDetectionScanner::new(
      build_info,
      build_meta,
      module_type,
      &mut vec![],
    ));
    program.visit_with_path(
      &mut ImportMetaScanner::new(
        &mut presentational_dependencies,
        resource_data,
        compiler_options,
      ),
      &mut Default::default(),
    );
  }

  (dependencies, presentational_dependencies, vec![])
}

pub fn scan_dependencies_with_string_replace(
  program: &Program,
  unresolved_mark: Mark,
  resource_data: &ResourceData,
  _compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  build_info: &mut BuildInfo,
  build_meta: &mut BuildMeta,
) -> ScanDependenciesResult {
  let mut dependencies: Vec<Box<dyn ModuleDependency>> = vec![];
  let presentational_dependencies: Vec<Box<dyn Dependency>> = vec![];
  let mut code_replace_source_dependencies: Vec<Box<dyn CodeReplaceSourceDependency>> = vec![];
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  let comments = program.comments.clone();

  program.visit_with(&mut ApiScanner::new(
    &unresolved_ctxt,
    resource_data,
    &mut code_replace_source_dependencies,
  ));

  if module_type.is_js_auto() || module_type.is_js_dynamic() {
    program.visit_with(&mut CommonJsImportDependencyScanner::new(
      &mut dependencies,
      &unresolved_ctxt,
    ));
    program.visit_with(&mut RequireContextScanner::new(&mut dependencies));
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
      &mut code_replace_source_dependencies,
    ));
    let mut import_map = Default::default();
    program.visit_with(&mut HarmonyImportDependencyScanner::new(
      &mut dependencies,
      &mut code_replace_source_dependencies,
      &mut import_map,
    ));
    program.visit_with(&mut HarmonyExportDependencyScanner::new(
      &mut dependencies,
      &mut code_replace_source_dependencies,
      &mut import_map,
    ));
  }

  (
    dependencies,
    presentational_dependencies,
    code_replace_source_dependencies,
  )
}
