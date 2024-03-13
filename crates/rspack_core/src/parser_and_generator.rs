use std::{collections::HashMap, fmt::Debug};

use derivative::Derivative;
use rkyv::AlignedVec;
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_loader_runner::{AdditionalData, ResourceData};
use rspack_sources::BoxSource;
use rspack_util::source_map::SourceMapKind;
use swc_core::common::Span;

use crate::ConcatenationScope;
use crate::{
  tree_shaking::visitor::OptimizeAnalyzeResult, AsyncDependenciesBlock, BoxDependency, BoxLoader,
  BuildExtraDataType, BuildInfo, BuildMeta, CodeGenerationData, Compilation, CompilerOptions,
  DependencyTemplate, GeneratorOptions, Module, ModuleDependency, ModuleIdentifier, ModuleType,
  ParserOptions, RuntimeGlobals, RuntimeSpec, SourceType,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ParseContext<'a> {
  pub source: BoxSource,
  pub module_identifier: ModuleIdentifier,
  pub module_type: &'a ModuleType,
  pub module_user_request: &'a str,
  pub module_parser_options: Option<&'a ParserOptions>,
  pub module_source_map_kind: SourceMapKind,
  #[derivative(Debug = "ignore")]
  pub loaders: &'a [BoxLoader],
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: AdditionalData,
  pub code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub build_info: &'a mut BuildInfo,
  pub build_meta: &'a mut BuildMeta,
}

#[derive(Debug)]
pub struct SideEffectsBailoutItem {
  pub msg: String,
  /// The type of AstNode
  pub ty: String,
}

impl SideEffectsBailoutItem {
  pub fn new(msg: String, ty: String) -> Self {
    Self { msg, ty }
  }
}

#[derive(Debug)]
pub struct SideEffectsBailoutItemWithSpan {
  pub span: Span,
  /// The type of AstNode
  pub ty: String,
}

impl SideEffectsBailoutItemWithSpan {
  pub fn new(span: Span, ty: String) -> Self {
    Self { span, ty }
  }
}

#[derive(Debug)]
pub struct ParseResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<AsyncDependenciesBlock>,
  pub presentational_dependencies: Vec<Box<dyn DependencyTemplate>>,
  pub source: BoxSource,
  pub analyze_result: OptimizeAnalyzeResult,
  pub side_effects_bailout: Option<SideEffectsBailoutItem>,
}

#[derive(Debug)]
pub struct GenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub module_generator_options: Option<&'a GeneratorOptions>,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub data: &'a mut CodeGenerationData,
  pub requested_source_type: SourceType,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<&'a mut ConcatenationScope>,
}

pub trait ParserAndGenerator: Send + Sync + Debug {
  /// The source types that the generator can generate (the source types you can make requests for)
  fn source_types(&self) -> &[SourceType];
  /// Parse the source and return the dependencies and the ast or source
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>>;
  /// Size of the original source
  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64;
  /// Generate source or AST based on the built source or AST
  fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource>;
  /// Store parser&generator data to cache
  fn store(&self, _extra_data: &mut HashMap<BuildExtraDataType, AlignedVec>) {}
  /// Resume parser&generator data from cache
  fn resume(&mut self, _extra_data: &HashMap<BuildExtraDataType, AlignedVec>) {}
}
