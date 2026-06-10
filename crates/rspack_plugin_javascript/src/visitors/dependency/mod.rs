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
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Comments, Program};
use swc_experimental_ecma_semantic::resolver::Semantic;

pub use self::{
  context_dependency_helper::{ContextModuleScanResult, create_context_dependency},
  parser::{
    AllowedMemberTypes, AtomMembers, CallExpressionInfo, CallHooksName,
    DestructuringAssignmentProperties, DestructuringAssignmentProperty, ExportedVariableInfo,
    JavascriptParser, MemberExpressionInfo, MemberRanges, OptionalMembers, PatRef, RootName,
    ScopeTerminated, TagInfoData, TopLevelScope, ast::*, estree::*,
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

pub struct ParsedJavaScriptAst<'ast> {
  pub allocator: &'ast Allocator,
  pub comments: &'ast Comments<'ast>,
  pub semantic: &'ast Semantic,
  pub program: &'ast Program<'ast>,
}

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  source: &str,
  ast: &ParsedJavaScriptAst<'_>,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  module_type: &ModuleType,
  module_layer: Option<&ModuleLayer>,
  factory_meta: Option<&FactoryMeta>,
  build_meta: &mut BuildMeta,
  build_info: &mut BuildInfo,
  module_identifier: ModuleIdentifier,
  module_parser_options: Option<&ParserOptions>,
  semicolons: &mut FxHashSet<u32>,
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
    &module_identifier,
    module_type,
    module_layer,
    resource_data,
    factory_meta,
    build_meta,
    build_info,
    semicolons,
    parser_plugins,
    parse_meta,
    parser_runtime_requirements,
  );

  parser.walk_program(ast.program);
  parser.into_results()
}
