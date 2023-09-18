mod api_scanner;
mod common_js_export_scanner;
mod common_js_import_dependency_scanner;
mod common_js_scanner;
mod compatibility_scanner;
mod context_helper;
mod export_info_api_scanner;
mod harmony_detection_scanner;
mod harmony_export_dependency_scanner;
mod harmony_import_dependency_scanner;
mod hot_module_replacement_scanner;
mod import_meta_scanner;
mod import_scanner;
mod node_stuff_scanner;
mod provide_scanner;
mod require_context_scanner;
mod url_scanner;
mod util;
mod worker_scanner;
use rspack_core::{
  ast::javascript::Program, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, ModuleIdentifier, ModuleType, ResourceData,
};
use swc_core::common::{comments::Comments, Mark, SyntaxContext};
pub use util::*;

use self::{
  api_scanner::ApiScanner, common_js_export_scanner::CommonJsExportDependencyScanner,
  common_js_import_dependency_scanner::CommonJsImportDependencyScanner,
  common_js_scanner::CommonJsScanner, compatibility_scanner::CompatibilityScanner,
  export_info_api_scanner::ExportInfoApiScanner,
  harmony_detection_scanner::HarmonyDetectionScanner,
  harmony_export_dependency_scanner::HarmonyExportDependencyScanner,
  harmony_import_dependency_scanner::HarmonyImportDependencyScanner,
  hot_module_replacement_scanner::HotModuleReplacementScanner,
  import_meta_scanner::ImportMetaScanner, import_scanner::ImportScanner,
  node_stuff_scanner::NodeStuffScanner, provide_scanner::ProvideScanner,
  require_context_scanner::RequireContextScanner, url_scanner::UrlScanner,
  worker_scanner::WorkerScanner,
};
pub type ScanDependenciesResult = (Vec<BoxDependency>, Vec<BoxDependencyTemplate>);

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
  let mut dependencies: Vec<BoxDependency> = vec![];
  let mut presentational_dependencies: Vec<BoxDependencyTemplate> = vec![];
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  let comments = program.comments.clone();
  let mut parser_exports_state = None;
  program.visit_with(&mut ApiScanner::new(
    &unresolved_ctxt,
    resource_data,
    &mut presentational_dependencies,
    compiler_options.output.module,
    build_info,
  ));

  program.visit_with(&mut CompatibilityScanner::new(
    &mut presentational_dependencies,
    &unresolved_ctxt,
  ));
  program.visit_with(&mut ExportInfoApiScanner::new(
    &mut presentational_dependencies,
    unresolved_ctxt,
  ));

  if !compiler_options.builtins.provide.is_empty() {
    program.visit_with(&mut ProvideScanner::new(
      &compiler_options.builtins.provide,
      &unresolved_ctxt,
      &mut dependencies,
    ));
  }

  if module_type.is_js_auto() || module_type.is_js_dynamic() {
    program.visit_with(&mut CommonJsScanner::new(
      &mut presentational_dependencies,
      &unresolved_ctxt,
    ));
    program.visit_with(&mut RequireContextScanner::new(&mut dependencies));
    program.visit_with(&mut CommonJsExportDependencyScanner::new(
      &mut presentational_dependencies,
      &unresolved_ctxt,
      build_meta,
      *module_type,
      &mut parser_exports_state,
    ));
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
      build_info,
    ));
    program.visit_with(&mut HarmonyExportDependencyScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      &mut import_map,
      build_info,
    ));
    let mut worker_syntax_scanner = rspack_core::needs_refactor::WorkerSyntaxScanner::new(
      rspack_core::needs_refactor::DEFAULT_WORKER_SYNTAX,
    );
    program.visit_with(&mut worker_syntax_scanner);
    let worker_syntax_list = &worker_syntax_scanner.into();
    let mut worker_scanner = WorkerScanner::new(
      &module_identifier,
      &compiler_options.output,
      worker_syntax_list,
    );
    program.visit_with(&mut worker_scanner);
    dependencies.append(&mut worker_scanner.dependencies);
    presentational_dependencies.append(&mut worker_scanner.presentational_dependencies);
    program.visit_with(&mut UrlScanner::new(&mut dependencies, worker_syntax_list));
    program.visit_with(&mut ImportMetaScanner::new(
      &mut presentational_dependencies,
      resource_data,
      compiler_options,
    ));
  }

  program.visit_with(&mut ImportScanner::new(
    &mut dependencies,
    comments.as_ref().map(|c| c as &dyn Comments),
    build_meta,
  ));

  if compiler_options.dev_server.hot {
    program.visit_with(&mut HotModuleReplacementScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      build_meta,
    ));
  }

  (dependencies, presentational_dependencies)
}
