use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::RspackHashDigest;
use rspack_macros::impl_source_map_config;
use rspack_sources::BoxSource;
use rspack_util::source_map::SourceMapKind;

use crate::{
  AsyncDependenciesBlockIdentifier, BoxModule, BuildContext, BuildInfo, BuildMeta, BuildResult,
  ChunkUkey, CodeGenerationResult, Compilation, Context, DependenciesBlock, DependencyId,
  FactoryMeta, LibIdentOptions, Module, ModuleCodeGenerationContext, ModuleGraph, ModuleIdentifier,
  ModuleType, RuntimeSpec, SourceType, impl_module_meta_info,
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct SelfModule {
  identifier: ModuleIdentifier,
  readable_identifier: String,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
}

impl SelfModule {
  pub fn new(module_identifier: ModuleIdentifier) -> Self {
    let identifier = format!("self {module_identifier}");
    Self {
      identifier: ModuleIdentifier::from(identifier.as_str()),
      readable_identifier: identifier,
      blocks: Default::default(),
      dependencies: Default::default(),
      factory_meta: None,
      build_info: BuildInfo {
        strict: true,
        ..Default::default()
      },
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for SelfModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for SelfModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait]
impl Module for SelfModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    self.identifier.len() as f64
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Fallback
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    self.readable_identifier.as_str().into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    None
  }

  fn chunk_condition(&self, _chunk: &ChunkUkey, _compilation: &Compilation) -> Option<bool> {
    None
  }

  // #[tracing::instrument("SelfModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    _code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    Ok(CodeGenerationResult::default())
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    // do nothing, since this is self reference, the module itself (parent module of this self module) should take effects
    Ok(RspackHashDigest::new(
      &[],
      &compilation.options.output.hash_digest,
    ))
  }

  async fn build(
    self: Box<Self>,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    Ok(BuildResult {
      module: BoxModule::new(self),
      dependencies: vec![],
      blocks: vec![],
      optimization_bailouts: vec![],
    })
  }
}

impl_empty_diagnosable_trait!(SelfModule);
