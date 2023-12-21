mod api_scanner;
mod common_js_export_scanner;
pub(crate) mod common_js_import_dependency_scanner;
mod common_js_scanner;
mod compatibility_scanner;
mod context_helper;
mod export_info_api_scanner;
mod harmony_detection_scanner;
mod harmony_export_dependency_scanner;
pub mod harmony_import_dependency_scanner;
mod harmony_top_level_this;
mod hot_module_replacement_scanner;
mod import_meta_scanner;
mod import_scanner;
mod node_stuff_scanner;
mod require_context_scanner;
mod url_scanner;
mod util;
mod worker_scanner;

use rspack_ast::javascript::Program;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, JavascriptParserUrl, ModuleIdentifier, ModuleType, ResourceData,
};
use rspack_error::miette::Diagnostic;
use rustc_hash::FxHashMap as HashMap;
use swc_core::common::Span;
use swc_core::common::{comments::Comments, Mark, SyntaxContext};
use swc_core::ecma::atoms::JsWord;
pub use util::*;

use self::harmony_import_dependency_scanner::ImportMap;
use self::{
  api_scanner::ApiScanner, common_js_export_scanner::CommonJsExportDependencyScanner,
  common_js_import_dependency_scanner::CommonJsImportDependencyScanner,
  common_js_scanner::CommonJsScanner, compatibility_scanner::CompatibilityScanner,
  export_info_api_scanner::ExportInfoApiScanner,
  harmony_detection_scanner::HarmonyDetectionScanner,
  harmony_export_dependency_scanner::HarmonyExportDependencyScanner,
  harmony_import_dependency_scanner::HarmonyImportDependencyScanner,
  harmony_top_level_this::HarmonyTopLevelThis,
  hot_module_replacement_scanner::HotModuleReplacementScanner,
  import_meta_scanner::ImportMetaScanner, import_scanner::ImportScanner,
  node_stuff_scanner::NodeStuffScanner, require_context_scanner::RequireContextScanner,
  url_scanner::UrlScanner, worker_scanner::WorkerScanner,
};

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<AsyncDependenciesBlock>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  // TODO: rename this name
  pub rewrite_usage_span: HashMap<Span, ExtraSpanInfo>,
  pub import_map: ImportMap,
  pub warning_diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>>,
}

#[derive(Debug, Clone, Default)]
pub enum ExtraSpanInfo {
  #[default]
  ReWriteUsedByExports,
  // (symbol, usage)
  // (local, exported) refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/javascript/JavascriptParser.js#L2347-L2352
  AddVariableUsage(Vec<(JsWord, JsWord)>),
}

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
) -> Result<ScanDependenciesResult, Vec<Box<dyn Diagnostic + Send + Sync>>> {
  let mut warning_diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>> = vec![];
  let mut errors = vec![];
  let mut dependencies = vec![];
  let mut blocks = vec![];
  let mut presentational_dependencies = vec![];
  let unresolved_ctxt = SyntaxContext::empty().apply_mark(unresolved_mark);
  let comments = program.comments.clone();
  let mut parser_exports_state = None;

  let mut rewrite_usage_span = HashMap::default();
  program.visit_with(&mut ApiScanner::new(
    unresolved_ctxt,
    resource_data,
    &mut dependencies,
    &mut presentational_dependencies,
    compiler_options.output.module,
    build_info,
  ));

  program.visit_with(&mut CompatibilityScanner::new(
    &mut presentational_dependencies,
    unresolved_ctxt,
  ));
  program.visit_with(&mut ExportInfoApiScanner::new(
    &mut presentational_dependencies,
    unresolved_ctxt,
  ));

  // TODO it should enable at js/auto or js/dynamic, but builtins provider will inject require at esm
  // https://github.com/web-infra-dev/rspack/issues/3544
  program.visit_with(&mut CommonJsImportDependencyScanner::new(
    &mut dependencies,
    &mut presentational_dependencies,
    unresolved_ctxt,
  ));
  if module_type.is_js_auto() || module_type.is_js_dynamic() {
    program.visit_with(&mut CommonJsScanner::new(
      &mut presentational_dependencies,
      unresolved_ctxt,
    ));
    program.visit_with(&mut RequireContextScanner::new(&mut dependencies));

    program.visit_with(&mut CommonJsExportDependencyScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      unresolved_ctxt,
      build_meta,
      *module_type,
      &mut parser_exports_state,
    ));
    if let Some(node_option) = &compiler_options.node {
      program.visit_with(&mut NodeStuffScanner::new(
        &mut presentational_dependencies,
        unresolved_ctxt,
        compiler_options,
        node_option,
        resource_data,
      ));
    }
  }

  let mut import_map = Default::default();

  if module_type.is_js_auto() || module_type.is_js_esm() {
    program.visit_with(&mut HarmonyDetectionScanner::new(
      &module_identifier,
      build_info,
      build_meta,
      module_type,
      compiler_options.experiments.top_level_await,
      &mut presentational_dependencies,
      &mut errors,
    ));
    program.visit_with(&mut HarmonyImportDependencyScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      &mut import_map,
      build_info,
      &mut rewrite_usage_span,
    ));
    let comments = program.comments.as_ref();
    program.visit_with(&mut HarmonyExportDependencyScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      &mut import_map,
      build_info,
      &mut rewrite_usage_span,
      comments,
    ));

    if build_meta.esm {
      program.visit_with(&mut HarmonyTopLevelThis {
        presentational_dependencies: &mut presentational_dependencies,
      })
    }

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
    blocks.append(&mut worker_scanner.blocks);
    dependencies.append(&mut worker_scanner.dependencies);
    presentational_dependencies.append(&mut worker_scanner.presentational_dependencies);

    let parse_url = &compiler_options
      .module
      .parser
      .as_ref()
      .and_then(|p| p.get(module_type))
      .and_then(|p| p.get_javascript(module_type))
      .map(|p| p.url)
      .unwrap_or(JavascriptParserUrl::Enable);

    if !matches!(parse_url, JavascriptParserUrl::Disable) {
      program.visit_with(&mut UrlScanner::new(
        &mut dependencies,
        worker_syntax_list,
        matches!(parse_url, JavascriptParserUrl::Relative),
      ));
    }
    program.visit_with(&mut ImportMetaScanner::new(
      &mut presentational_dependencies,
      resource_data,
      compiler_options,
      &mut warning_diagnostics,
    ));
  }

  program.visit_with(&mut ImportScanner::new(
    module_identifier,
    &mut dependencies,
    &mut blocks,
    comments.as_ref().map(|c| c as &dyn Comments),
    build_meta,
    compiler_options
      .module
      .parser
      .as_ref()
      .and_then(|p| p.get(module_type))
      .and_then(|p| p.get_javascript(module_type)),
  ));

  if compiler_options.dev_server.hot {
    program.visit_with(&mut HotModuleReplacementScanner::new(
      &mut dependencies,
      &mut presentational_dependencies,
      build_meta,
    ));
  }

  if errors.is_empty() {
    Ok(ScanDependenciesResult {
      dependencies,
      blocks,
      presentational_dependencies,
      rewrite_usage_span,
      import_map,
      warning_diagnostics,
    })
  } else {
    Err(errors)
  }
}
