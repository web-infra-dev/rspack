use std::{borrow::Cow, hash::Hash};

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset},
};
use rspack_collections::{Identifiable, IdentifierMap, IdentifierSet};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_macros::impl_source_map_config;
use rspack_sources::{BoxSource, OriginalSource, RawStringSource, SourceExt};
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

use crate::{
  BuildInfo, BuildMeta, CodeGenerationResult, Compilation, ConnectionState, Context,
  DependenciesBlock, DependencyId, FactoryMeta, Module, ModuleCodeGenerationContext, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
  dependencies_block::AsyncDependenciesBlockIdentifier, impl_module_meta_info, module_update_hash,
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct RawModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  source_str: String,
  #[cacheable(with=AsOption<AsPreset>)]
  source: Option<BoxSource>,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  runtime_requirements: RuntimeGlobals,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
}

static RAW_MODULE_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];

impl RawModule {
  pub fn new(
    source_str: String,
    identifier: ModuleIdentifier,
    readable_identifier: String,
    runtime_requirements: RuntimeGlobals,
  ) -> Self {
    Self {
      blocks: Default::default(),
      dependencies: Default::default(),
      source_str,
      source: None,
      identifier,
      readable_identifier,
      runtime_requirements,
      factory_meta: None,
      build_info: BuildInfo {
        cacheable: true,
        strict: true,
        ..Default::default()
      },
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for RawModule {
  fn identifier(&self) -> ModuleIdentifier {
    self.identifier
  }
}

impl DependenciesBlock for RawModule {
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
#[async_trait::async_trait]
impl Module for RawModule {
  impl_module_meta_info!();

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsAuto
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    RAW_MODULE_SOURCE_TYPES
  }

  fn source(&self) -> Option<&BoxSource> {
    self.source.as_ref()
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    Cow::Borrowed(&self.readable_identifier)
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    f64::max(1.0, self.source_str.len() as f64)
  }

  // #[tracing::instrument("RawModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    code_generation_context
      .runtime_template
      .runtime_requirements_mut()
      .insert(self.runtime_requirements);
    if self.get_source_map_kind().enabled() {
      cgr.add(
        SourceType::JavaScript,
        OriginalSource::new(self.source_str.clone(), self.identifier.to_string()).boxed(),
      );
    } else {
      cgr.add(
        SourceType::JavaScript,
        RawStringSource::from(self.source_str.clone()).boxed(),
      );
    };
    Ok(cgr)
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    self.source_str.hash(&mut hasher);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }

  fn get_side_effects_connection_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    if let Some(side_effect_free) = self.factory_meta().and_then(|m| m.side_effect_free) {
      return ConnectionState::Active(!side_effect_free);
    }
    ConnectionState::Active(true)
  }
}

impl_empty_diagnosable_trait!(RawModule);
