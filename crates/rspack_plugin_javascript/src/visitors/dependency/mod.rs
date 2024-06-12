mod context_dependency_helper;
mod context_helper;
mod parser;
mod util;

use rspack_ast::javascript::Program;
use rspack_core::needs_refactor::WorkerSyntaxList;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, ParserOptions,
};
use rspack_core::{BuildMeta, CompilerOptions, ModuleIdentifier, ModuleType, ResourceData};
use rspack_error::miette::Diagnostic;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::comments::Comments;
use swc_core::common::{BytePos, SourceFile, Span};
use swc_core::ecma::atoms::Atom;

pub use self::context_dependency_helper::create_context_dependency;
pub use self::context_helper::{scanner_context_module, ContextModuleScanResult};
pub use self::parser::{
  AllowedMemberTypes, CallExpressionInfo, CallHooksName, ExportedVariableInfo, PathIgnoredSpans,
};
pub use self::parser::{JavascriptParser, MemberExpressionInfo, TagInfoData, TopLevelScope};
pub use self::util::*;
use crate::dependency::Specifier;

#[derive(Debug)]
pub struct ImporterReferenceInfo {
  pub request: Atom,
  pub specifier: Specifier,
  pub names: Option<Atom>,
  pub source_order: i32,
}

impl ImporterReferenceInfo {
  pub fn new(request: Atom, specifier: Specifier, names: Option<Atom>, source_order: i32) -> Self {
    Self {
      request,
      specifier,
      names,
      source_order,
    }
  }
}

pub type ImportMap = FxHashMap<swc_core::ecma::ast::Id, ImporterReferenceInfo>;

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<AsyncDependenciesBlock>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub usage_span_record: FxHashMap<Span, ExtraSpanInfo>,
  pub import_map: ImportMap,
  pub warning_diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>>,
}

#[derive(Debug, Clone, Default)]
pub enum ExtraSpanInfo {
  #[default]
  ReWriteUsedByExports,
  // (symbol, usage)
  // (local, exported) refer https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/javascript/JavascriptParser.js#L2347-L2352
  AddVariableUsage(Vec<(Atom, Atom)>),
}

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  source_file: &SourceFile,
  program: &Program,
  worker_syntax_list: &mut WorkerSyntaxList,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  build_info: &mut BuildInfo,
  build_meta: &mut BuildMeta,
  module_identifier: ModuleIdentifier,
  module_parser_options: Option<&ParserOptions>,
  semicolons: &mut FxHashSet<BytePos>,
  path_ignored_spans: &mut PathIgnoredSpans,
) -> Result<ScanDependenciesResult, Vec<Box<dyn Diagnostic + Send + Sync>>> {
  let mut parser = JavascriptParser::new(
    source_file,
    compiler_options,
    module_parser_options
      .and_then(|p| p.get_javascript())
      .expect("should at least have a global javascript parser options"),
    program.comments.as_ref().map(|c| c as &dyn Comments),
    &module_identifier,
    module_type,
    worker_syntax_list,
    resource_data,
    build_meta,
    build_info,
    semicolons,
    path_ignored_spans,
  );

  parser.walk_program(program.get_inner_program());

  if parser.errors.is_empty() {
    Ok(ScanDependenciesResult {
      dependencies: parser.dependencies,
      blocks: parser.blocks,
      presentational_dependencies: parser.presentational_dependencies,
      usage_span_record: parser.rewrite_usage_span,
      import_map: parser.import_map,
      warning_diagnostics: parser.warning_diagnostics,
    })
  } else {
    Err(parser.errors)
  }
}
