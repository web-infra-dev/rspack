use std::borrow::Cow;

use rspack_cacheable::{cacheable, cacheable_dyn, utils::OwnedOrRef};
use rspack_collections::Identifiable;
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::RspackHashDigest;
use rspack_sources::BoxSource;
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

use crate::{
  AsyncDependenciesBlockIdentifier, BoxModule, BuildInfo, BuildMeta, CodeGenerationResult,
  Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId, FactoryMeta, Module,
  ModuleGraph, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType, ValueCacheVersions,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct TempModule {
  id: ModuleIdentifier,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
}

impl TempModule {
  pub fn transform_from(module: OwnedOrRef<BoxModule>) -> OwnedOrRef<BoxModule> {
    let m = module.as_ref();
    OwnedOrRef::Owned(BoxModule::new(Box::new(Self {
      id: m.identifier(),
      build_info: m.build_info().clone(),
      build_meta: m.build_meta().clone(),
      dependencies: m.get_dependencies().to_vec(),
      // clean all of blocks
      blocks: vec![],
    })))
  }
}

impl_empty_diagnosable_trait!(TempModule);

impl ModuleSourceMapConfig for TempModule {
  fn get_source_map_kind(&self) -> &SourceMapKind {
    unreachable!()
  }

  fn set_source_map_kind(&mut self, _source_map: SourceMapKind) {
    unreachable!()
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for TempModule {
  fn factory_meta(&self) -> Option<&FactoryMeta> {
    unreachable!()
  }

  fn set_factory_meta(&mut self, _factory_meta: FactoryMeta) {
    unreachable!()
  }

  fn build_info(&self) -> &BuildInfo {
    &self.build_info
  }

  fn build_info_mut(&mut self) -> &mut BuildInfo {
    &mut self.build_info
  }

  fn build_meta(&self) -> &BuildMeta {
    &self.build_meta
  }

  fn build_meta_mut(&mut self) -> &mut BuildMeta {
    &mut self.build_meta
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    unreachable!()
  }

  fn module_type(&self) -> &ModuleType {
    unreachable!()
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    unreachable!()
  }

  fn source(&self) -> Option<&BoxSource> {
    unreachable!()
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    unreachable!()
  }

  fn need_build(&self, _value_cache_versions: &ValueCacheVersions) -> bool {
    // return true to make sure this module always rebuild
    true
  }

  async fn code_generation(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    unreachable!()
  }

  async fn get_runtime_hash(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    unreachable!()
  }
}

impl Identifiable for TempModule {
  fn identifier(&self) -> rspack_collections::Identifier {
    self.id
  }
}

impl DependenciesBlock for TempModule {
  fn add_block_id(&mut self, _block: AsyncDependenciesBlockIdentifier) {
    unreachable!()
  }
  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }
  fn add_dependency_id(&mut self, _dependency: DependencyId) {
    unreachable!()
  }
  fn remove_dependency_id(&mut self, _dependency: DependencyId) {
    unreachable!()
  }
  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}
