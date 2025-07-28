mod context_dependency_helper;
mod parser;
mod util;

use std::sync::Arc;

use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, FactoryMeta, ModuleIdentifier, ModuleLayer, ModuleType, ParseMeta,
  ParserOptions, ResourceData,
};
use rspack_error::miette::Diagnostic;
use rspack_javascript_compiler::ast::Program;
use rustc_hash::FxHashSet;
use swc_core::common::{comments::Comments, BytePos, Mark, SourceFile, SourceMap};

pub use self::{
  context_dependency_helper::{create_context_dependency, ContextModuleScanResult},
  parser::{
    estree::*, AllowedMemberTypes, CallExpressionInfo, CallHooksName,
    DestructuringAssignmentProperty, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo,
    RootName, TagInfoData, TopLevelScope,
  },
  util::*,
};
use crate::BoxJavascriptParserPlugin;

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub warning_diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>>,
}

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  source_map: Arc<SourceMap>,
  source_file: &SourceFile,
  program: &Program,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  module_layer: Option<&ModuleLayer>,
  factory_meta: Option<&FactoryMeta>,
  build_meta: &mut BuildMeta,
  build_info: &mut BuildInfo,
  module_identifier: ModuleIdentifier,
  module_parser_options: Option<&ParserOptions>,
  semicolons: &mut FxHashSet<BytePos>,
  unresolved_mark: Mark,
  parser_plugins: &mut Vec<BoxJavascriptParserPlugin>,
  parse_meta: ParseMeta,
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
    module_layer,
    resource_data,
    factory_meta,
    build_meta,
    build_info,
    semicolons,
    unresolved_mark,
    parser_plugins,
    parse_meta,
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
