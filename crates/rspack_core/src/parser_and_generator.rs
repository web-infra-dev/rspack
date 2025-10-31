use std::{any::Any, borrow::Cow, ops::Deref};

use derive_more::with_trait::Debug;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsMap, AsPreset, AsVec},
};
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_hash::RspackHashDigest;
use rspack_loader_runner::{AdditionalData, ParseMeta, ResourceData};
use rspack_sources::BoxSource;
use rspack_util::{ext::AsAny, source_map::SourceMapKind};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::{atoms::Atom, common::Span};

use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BoxLoader, BoxModuleDependency,
  BuildInfo, BuildMeta, ChunkGraph, CodeGenerationData, Compilation, CompilerOptions,
  ConcatenationScope, Context, EvaluatedInlinableValue, FactoryMeta, Module, ModuleGraph,
  ModuleIdentifier, ModuleLayer, ModuleType, NormalModule, ParserOptions, RuntimeGlobals,
  RuntimeSpec, SourceType,
};

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
  pub module_match_resource: Option<&'a ResourceData>,
  #[debug(skip)]
  pub loaders: &'a [BoxLoader],
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: Option<AdditionalData>,
  pub factory_meta: Option<&'a FactoryMeta>,
  pub parse_meta: ParseMeta,
  pub build_info: &'a mut BuildInfo,
  pub build_meta: &'a mut BuildMeta,
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CollectedTypeScriptInfo {
  #[cacheable(with=AsVec<AsPreset>)]
  pub type_exports: FxHashSet<Atom>,
  #[cacheable(with=AsMap<AsPreset>)]
  pub exported_enums: FxHashMap<Atom, TSEnumValue>,
}

pub const COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY: &str = "rspack-collected-ts-info";

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct TSEnumValue(
  #[cacheable(with=AsMap<AsPreset>)] FxHashMap<Atom, Option<EvaluatedInlinableValue>>,
);

impl TSEnumValue {
  pub fn new(value: FxHashMap<Atom, Option<EvaluatedInlinableValue>>) -> Self {
    Self(value)
  }
}

impl Deref for TSEnumValue {
  type Target = FxHashMap<Atom, Option<EvaluatedInlinableValue>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
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
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub code_generation_dependencies: Vec<BoxModuleDependency>,
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
#[async_trait::async_trait]
pub trait ParserAndGenerator: Send + Sync + Debug + AsAny {
  /// The source types that the generator can generate (the source types you can make requests for)
  fn source_types(&self, module: &dyn Module, module_graph: &ModuleGraph) -> &[SourceType];
  /// Parse the source and return the dependencies and the ast or source
  async fn parse<'a>(
    &mut self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>>;
  /// Size of the original source
  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64;
  /// Generate source or AST based on the built source or AST
  async fn generate(
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

  async fn get_runtime_hash(
    &self,
    _module: &NormalModule,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    Ok(RspackHashDigest::new(
      &[],
      &compilation.options.output.hash_digest,
    ))
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
