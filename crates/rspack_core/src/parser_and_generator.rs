use std::any::Any;
use std::borrow::Cow;

use derive_more::Debug;
use rspack_cacheable::cacheable_dyn;
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_loader_runner::{AdditionalData, ResourceData};
use rspack_sources::BoxSource;
use rspack_util::ext::AsAny;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap;
use swc_core::common::Span;

use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxLoader, BuildInfo, BuildMeta, CodeGenerationData,
  Compilation, CompilerOptions, DependencyTemplate, Module, ModuleDependency, ModuleIdentifier,
  ModuleLayer, ModuleType, NormalModule, ParserOptions, RuntimeGlobals, RuntimeSpec, SourceType,
};
use crate::{ChunkGraph, ConcatenationScope, Context, ModuleGraph};

#[derive(Debug)]
pub struct ParseContext<'a> {
  pub source: BoxSource,
  pub module_context: &'a Context,
  pub module_identifier: ModuleIdentifier,
  pub module_type: &'a ModuleType,
  pub module_layer: Option<&'a ModuleLayer>,
  pub module_user_request: &'a str,
  pub module_parser_options: Option<&'a ParserOptions>,
  pub module_source_map_kind: SourceMapKind,
  #[debug(skip)]
  pub loaders: &'a [BoxLoader],
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: Option<AdditionalData>,
  pub parse_meta: FxHashMap<String, String>,
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
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<Box<dyn DependencyTemplate>>,
  pub code_generation_dependencies: Vec<Box<dyn ModuleDependency>>,
  pub source: BoxSource,
  pub side_effects_bailout: Option<SideEffectsBailoutItem>,
}

#[derive(Debug)]
pub struct GenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub data: &'a mut CodeGenerationData,
  pub requested_source_type: SourceType,
  pub runtime: Option<&'a RuntimeSpec>,
  pub concatenation_scope: Option<&'a mut ConcatenationScope>,
}

#[cacheable_dyn]
pub trait ParserAndGenerator: Send + Sync + Debug + AsAny {
  /// The source types that the generator can generate (the source types you can make requests for)
  fn source_types(&self) -> &[SourceType];
  /// Parse the source and return the dependencies and the ast or source
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>>;
  /// Size of the original source
  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64;
  /// Generate source or AST based on the built source or AST
  fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource>;

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>>;

  fn update_hash(
    &self,
    _module: &NormalModule,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    Ok(())
  }
}

impl dyn ParserAndGenerator + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }

  pub fn is<D: Any>(&self) -> bool {
    self.downcast_ref::<D>().is_some()
  }
}
