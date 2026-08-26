mod context_dependency_helper;
mod parser;
mod util;

use rspack_core::{
  ArcComputed, AsyncDependenciesBlock, BoxDependency, BuildInfo, BuildMeta, CompilerOptions,
  DependencyCodeGenerationRef, FactoryMeta, ImportMeta, ModuleIdentifier, ModuleLayer, ModuleType,
  ParseMeta, ParserOptions, ResolvedModuleOptions, ResourceData, SideEffectsBailoutItemWithSpan,
};
use rspack_error::Diagnostic;
use rspack_util::swc::RspackComments;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{Ast, Program};

pub(crate) use self::parser::{StatementPath, member_property_key_to_atom};
pub use self::{
  context_dependency_helper::{ContextModuleScanResult, create_context_dependency},
  parser::{
    AllowedMemberTypes, AtomMembers, CallExpressionInfo, CallHooksName,
    DestructuringAssignmentProperties, DestructuringAssignmentProperty, ExportedVariableInfo,
    ExpressionExpressionInfo, JavascriptParser, MemberExpressionInfo, MemberRanges,
    OptionalMembers, PatRef, RootName, ScopeTerminated, TagInfoData, TopLevelScope, ast::*,
    estree::*,
  },
  util::*,
};
use crate::{
  BoxJavascriptParserPlugin, parser_and_generator::ParserRuntimeRequirementsData,
  visitors::name_resolution::JavascriptNameResolution,
};

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<DependencyCodeGenerationRef>,
  pub warning_diagnostics: Vec<Diagnostic>,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
}

pub struct ParsedJavaScriptAst<'ast> {
  pub ast: &'ast Ast<'ast>,
  pub comments: &'ast RspackComments<'ast>,
  pub name_resolution: &'ast JavascriptNameResolution<'ast>,
  pub program: Program,
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
  import_meta: ArcComputed<ResolvedModuleOptions, ImportMeta>,
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
    import_meta,
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
