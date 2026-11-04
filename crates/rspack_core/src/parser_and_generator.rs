use std::{any::Any, borrow::Cow, ops::Deref};

use derive_more::with_trait::Debug;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsMap, AsPreset, AsVec},
};
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_hash::RspackHashDigest;
use rspack_intern::Atom;
use rspack_loader_runner::{AdditionalData, ParseMeta, ParseMetaValue, ResourceData};
use rspack_paths::Utf8PathBuf;
use rspack_sources::BoxSource;
use rspack_util::{ext::AsAny, source_map::SourceMapKind};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxLoader, BuildInfo, BuildMeta, ChunkGraph,
  CodeGenerationData, Compilation, CompilerOptions, ConcatenationScope, Context,
  DependencyCodeGenerationRef, DependencyId, DependencyLocation, DependencyRange,
  EvaluatedInlinableValue, FactoryMeta, GeneratorOptions, Module, ModuleCodeTemplate, ModuleGraph,
  ModuleIdentifier, ModuleLayer, ModuleType, NormalModule, ParserOptions, PortableRoot,
  RuntimeSpec, SourceType,
};

/// A public source and the byte replacements that relate it to the stored parser source.
/// Kept outside the persistent cache: the public spelling can depend on the current build.
#[derive(Debug)]
pub struct SourceView {
  pub source: BoxSource,
  /// Sorted, disjoint stored ranges paired with their rendered byte lengths.
  replacements: Vec<(DependencyRange, u32)>,
}

impl SourceView {
  pub fn new(source: BoxSource, replacements: Vec<(DependencyRange, u32)>) -> Self {
    Self {
      source,
      replacements,
    }
  }

  fn offset(&self, offset: usize, end: bool) -> usize {
    let mut delta = 0i64;
    for (range, length) in &self.replacements {
      if offset <= range.start as usize {
        break;
      }
      if offset < range.end as usize {
        // A replaced interior has no exact counterpart. Snap outward to a replacement boundary,
        // keeping ranges valid even when the replacement has different UTF-8 characters.
        return (i64::from(range.start) + delta + if end { i64::from(*length) } else { 0 })
          as usize;
      }
      delta += i64::from(*length) - i64::from(range.end - range.start);
    }
    (offset as i64 + delta) as usize
  }

  /// Map a half-open byte range. Empty ranges stay empty, including inside a replacement.
  pub fn range(&self, start: usize, end: usize) -> (usize, usize) {
    (self.offset(start, false), self.offset(end, start != end))
  }

  pub fn dependency_range(&self, range: DependencyRange) -> DependencyRange {
    let (start, end) = self.range(range.start as usize, range.end as usize);
    DependencyRange::new(start as u32, end as u32)
  }
}

#[derive(Debug)]
pub struct ParseContext<'a> {
  pub source: BoxSource,
  pub module_context: &'a Context,
  pub module_identifier: ModuleIdentifier,
  pub module_type: &'a ModuleType,
  pub module_layer: Option<&'a ModuleLayer>,
  pub module_user_request: &'a str,
  pub module_parser_options: Option<&'a ParserOptions>,
  pub module_generator_options: Option<&'a GeneratorOptions>,
  pub module_source_map_kind: SourceMapKind,
  pub module_match_resource: Option<&'a ResourceData>,
  #[debug(skip)]
  pub loaders: &'a [BoxLoader],
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: Option<AdditionalData>,
  pub factory_meta: Option<&'a FactoryMeta>,
  pub parse_meta: ParseMeta,
  /// Absolute path values that every content-producing loader invocation certified as visible
  /// project paths, or `None` when the certification chain is broken. Only parsers that can
  /// relocate whole literals read it; the loader output itself still holds the real paths.
  pub portable_output: Option<&'a [Utf8PathBuf]>,
  /// The portable root, when the loader chain generated the root paths in `source`. Parsers may
  /// rewrite absolute module requests below it, which move with the checkout.
  pub portable_requests: Option<&'a PortableRoot>,
  pub build_info: &'a mut BuildInfo,
  pub build_meta: &'a mut BuildMeta,
  pub runtime_template: &'a ModuleCodeTemplate,
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CollectedTypeScriptInfo {
  #[cacheable(with=AsVec<AsPreset>)]
  pub type_exports: FxHashSet<Atom>,
  #[cacheable(with=AsMap<AsPreset>)]
  pub exported_enums: FxHashMap<Atom, TSEnumValue>,
}

#[cacheable_dyn]
impl ParseMetaValue for CollectedTypeScriptInfo {
  fn clone_parse_meta(&self) -> Box<dyn ParseMetaValue> {
    Box::new(self.clone())
  }

  fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
    self
  }
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
  pub loc: DependencyLocation,
  /// The type of AstNode
  pub ty: String,
}

impl SideEffectsBailoutItem {}

#[derive(Debug)]
pub struct SideEffectsBailoutItemWithSpan {
  pub range: DependencyRange,
  pub loc: Option<DependencyLocation>,
  /// The type of AstNode
  pub ty: String,
}

impl SideEffectsBailoutItemWithSpan {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>, ty: String) -> Self {
    Self { range, loc, ty }
  }
}

#[derive(Debug)]
pub struct ParseResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<DependencyCodeGenerationRef>,
  pub code_generation_dependencies: Vec<DependencyId>,
  pub source: BoxSource,
  pub side_effects_bailout: Option<SideEffectsBailoutItem>,
}

#[derive(Debug)]
pub struct GenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub runtime_template: &'a mut ModuleCodeTemplate,
  pub data: &'a mut CodeGenerationData,
  pub requested_source_type: SourceType,
  pub module_parser_options: Option<&'a ParserOptions>,
  pub module_generator_options: Option<&'a GeneratorOptions>,
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

  fn has_runtime_hash(&self) -> bool {
    false
  }

  /// Renders the public view of a stored source that this parser normalized for code generation
  /// and the build hash, such as root-free canonical literals. `None` means the stored source is
  /// already the public view.
  fn source_view(
    &self,
    _source: &BoxSource,
    _presentational_dependencies: &[DependencyCodeGenerationRef],
  ) -> Option<SourceView> {
    None
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
