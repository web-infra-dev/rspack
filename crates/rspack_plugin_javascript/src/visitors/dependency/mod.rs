mod context_dependency_helper;
mod parser;
mod util;

use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, FactoryMeta, ModuleIdentifier, ModuleLayer, ModuleType, ParseMeta,
  ParserOptions, ResourceData, SideEffectsBailoutItemWithSpan,
};
use rspack_error::Diagnostic;
use rustc_hash::FxHashSet;
use swc_core::common::{BytePos, comments::Comments};
use swc_experimental_ecma_ast::{Ast, Program};
use swc_experimental_ecma_semantic::ScopeId;
use swc_node_comments::SwcComments;

pub use self::{
  context_dependency_helper::{ContextModuleScanResult, create_context_dependency},
  parser::{
    AllowedMemberTypes, CallExpressionInfo, CallHooksName, DestructuringAssignmentProperties,
    DestructuringAssignmentProperty, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo,
    RootName, TagInfoData, TopLevelScope, estree::*,
  },
  util::*,
};
use crate::{BoxJavascriptParserPlugin, parser_and_generator::ParserRuntimeRequirementsData};

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub warning_diagnostics: Vec<Diagnostic>,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
}

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  source: &str,
  ast: Ast,
  program: Program,
  comments: Option<SwcComments>,
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
  unresolved_scope_id: ScopeId,
  parser_plugins: &mut Vec<BoxJavascriptParserPlugin>,
  parse_meta: ParseMeta,
  parser_runtime_requirements: &ParserRuntimeRequirementsData,
) -> Result<ScanDependenciesResult, Vec<Diagnostic>> {
  let mut parser = JavascriptParser::new(
    source,
    ast,
    compiler_options,
    module_parser_options
      .and_then(|p| p.get_javascript())
      .expect("should at least have a global javascript parser options"),
    comments.as_ref().map(|c| c as &dyn Comments),
    &module_identifier,
    module_type,
    module_layer,
    resource_data,
    factory_meta,
    build_meta,
    build_info,
    semicolons,
    unresolved_scope_id,
    parser_plugins,
    parse_meta,
    parser_runtime_requirements,
  );

  parser.walk_program(program);
  parser.into_results()
}
