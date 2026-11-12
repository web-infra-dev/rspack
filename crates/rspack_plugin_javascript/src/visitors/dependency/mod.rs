mod context_dependency_helper;
mod parser;
mod util;

use rspack_core::{
  ArcComputed, AsyncDependenciesBlock, BoxDependency, BuildInfo, BuildMeta, CompilerOptions,
  DependencyCodeGenerationRef, DependencyRange, FactoryMeta, ImportMeta, ModuleIdentifier,
  ModuleLayer, ModuleType, ParseMeta, ParserOptions, PortableLocationShift, ResolvedModuleOptions,
  ResourceData, SideEffectsBailoutItemWithSpan,
};
use rspack_error::Diagnostic;
use rspack_util::swc::RspackComments;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{Ast, Program};
use swc_next_ecma_semantic::Semantic;

pub(crate) use self::parser::{StatementPath, member_property_key_to_atom};
pub use self::{
  context_dependency_helper::{ContextModuleScanResult, create_context_dependency},
  parser::{
    AllowedMemberTypes, AtomMembers, CallExpressionInfo, CallHooksName,
    DestructuringAssignmentProperties, DestructuringAssignmentProperty, ExportedVariableInfo,
    ExpressionExpressionInfo, JavascriptParser, MemberExpressionInfo, MemberRanges,
    OptionalMembers, PatRef, RootName, ScopeTerminated, TagInfoData, TopLevelScope, estree::*,
  },
  util::*,
};
use crate::{BoxJavascriptParserPlugin, parser_and_generator::ParserRuntimeRequirementsData};

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<DependencyCodeGenerationRef>,
  pub warning_diagnostics: Vec<Diagnostic>,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
  /// Whether the evaluator read a certified path literal.
  pub portable_site_evaluated: bool,
  /// Whether a dependency range cuts into a certified path literal.
  pub portable_site_cut: bool,
}

pub struct ParsedJavaScriptAst<'ast> {
  pub ast: &'ast Ast<'ast>,
  pub comments: &'ast RspackComments<'ast>,
  pub semantic: &'ast Semantic<'ast>,
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
  portable_sites: Option<&[DependencyRange]>,
  portable_location_shifts: Option<&[PortableLocationShift]>,
) -> Result<ScanDependenciesResult, Vec<Diagnostic>> {
  let javascript_options = module_parser_options
    .and_then(|p| p.get_javascript())
    .expect("should at least have a global javascript parser options");
  // The drive is owned here and borrowed by the parser: hooks receive
  // `&mut parser`, so a parser that owned its drive could not lend it out
  // without cloning the handle on every hook call.
  let module_plugin_count = parser_plugins.len();
  let plugin_drive = JavascriptParser::build_plugin_drive(
    compiler_options,
    javascript_options,
    module_type,
    import_meta,
    build_info,
    parser_plugins,
  );
  let mut parser = JavascriptParser::new(
    source,
    ast,
    compiler_options,
    javascript_options,
    &plugin_drive,
    &module_identifier,
    module_type,
    module_layer,
    resource_data,
    factory_meta,
    build_meta,
    build_info,
    semicolons,
    parse_meta,
    parser_runtime_requirements,
  );

  if let Some(portable_sites) = portable_sites {
    parser.portable_sites = portable_sites;
  }
  if let Some(shifts) = portable_location_shifts {
    parser.portable_location_shifts = shifts;
  }

  parser.walk_program(ast.program);
  let result = parser.into_results();
  if portable_sites.is_some() {
    // The caller may scan the real text again with the same module plugins.
    parser_plugins.extend(
      plugin_drive
        .into_plugins()
        .into_iter()
        .take(module_plugin_count),
    );
  }
  result
}
