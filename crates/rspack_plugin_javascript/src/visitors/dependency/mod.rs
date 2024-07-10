mod context_dependency_helper;
mod parser;
mod util;

use std::sync::Arc;

use rspack_ast::javascript::Program;
use rspack_core::{
  AdditionalData, AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo,
  ParserOptions,
};
use rspack_core::{BuildMeta, CompilerOptions, ModuleIdentifier, ModuleType, ResourceData};
use rspack_error::miette::Diagnostic;
use rustc_hash::FxHashSet;
use swc_core::common::Mark;
use swc_core::common::{comments::Comments, BytePos, SourceFile, SourceMap};
use swc_core::ecma::atoms::Atom;

pub use self::context_dependency_helper::{create_context_dependency, ContextModuleScanResult};
pub use self::parser::{
  estree::*, AllowedMemberTypes, CallExpressionInfo, CallHooksName, ExportedVariableInfo,
  JavascriptParser, MemberExpressionInfo, TagInfoData, TopLevelScope,
};
pub use self::util::*;
use crate::BoxJavascriptParserPlugin;

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<AsyncDependenciesBlock>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
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
  source_map: Arc<SourceMap>,
  source_file: &SourceFile,
  program: &Program,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  build_info: &mut BuildInfo,
  build_meta: &mut BuildMeta,
  module_identifier: ModuleIdentifier,
  module_parser_options: Option<&ParserOptions>,
  semicolons: &mut FxHashSet<BytePos>,
  unresolved_mark: Mark,
  parser_plugins: &mut Vec<BoxJavascriptParserPlugin>,
  additional_data: AdditionalData,
) -> Result<ScanDependenciesResult, Vec<Box<dyn Diagnostic + Send + Sync>>> {
  let mut parser = JavascriptParser::new(
    source_map,
    source_file,
    compiler_options,
    module_parser_options
      .and_then(|p| p.get_javascript())
      .expect("should at least have a global javascript parser options"),
    program.comments.as_ref().map(|c| c as &dyn Comments),
    &module_identifier,
    module_type,
    resource_data,
    build_meta,
    build_info,
    semicolons,
    unresolved_mark,
    parser_plugins,
    additional_data,
  );

  parser.walk_program(program.get_inner_program());

  if parser.errors.is_empty() {
    Ok(ScanDependenciesResult {
      dependencies: parser.dependencies,
      blocks: parser.blocks,
      presentational_dependencies: parser.presentational_dependencies,
      warning_diagnostics: parser.warning_diagnostics,
    })
  } else {
    Err(parser.errors)
  }
}
