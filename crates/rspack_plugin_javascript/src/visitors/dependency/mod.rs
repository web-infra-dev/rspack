mod context_dependency_helper;
mod parser;
mod util;

use std::sync::Arc;

use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, FactoryMeta, ModuleIdentifier, ModuleLayer, ModuleType, ParseMeta,
  ParserOptions, ResourceData, RuntimeTemplate, SideEffectsBailoutItemWithSpan,
};
use rspack_error::Diagnostic;
use rspack_javascript_compiler::ast::Program;
use rustc_hash::FxHashSet;
use swc_core::common::{BytePos, Mark, SourceMap, comments::Comments};

pub use self::{
  context_dependency_helper::{ContextModuleScanResult, create_context_dependency},
  parser::{
    AllowedMemberTypes, CallExpressionInfo, CallHooksName, DestructuringAssignmentProperties,
    DestructuringAssignmentProperty, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo,
    RootName, TagInfoData, TopLevelScope, estree::*,
  },
  util::*,
};
use crate::BoxJavascriptParserPlugin;

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub warning_diagnostics: Vec<Diagnostic>,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
}

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  source_map: Arc<SourceMap>,
  source: &str,
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
  runtime_template: &RuntimeTemplate,
) -> Result<ScanDependenciesResult, Vec<Diagnostic>> {
  let mut parser = JavascriptParser::new(
    source_map,
    source,
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
    runtime_template,
  );

  parser.walk_program(program.get_inner_program());
  parser.into_results()
}
