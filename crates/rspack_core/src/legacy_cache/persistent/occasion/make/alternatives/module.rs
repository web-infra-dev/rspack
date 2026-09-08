use std::borrow::Cow;

use rspack_cacheable::{cacheable, cacheable_dyn, utils::OwnedOrRef};
use rspack_collections::Identifiable;
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::RspackHashDigest;
use rspack_sources::BoxSource;
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

use crate::{
  BoxModule, BuildContext, BuildInfo, BuildMeta, CodeGenerationResultBuilder, Compilation, Context,
  DependenciesBlock, DependenciesBlockData, FactoryMeta, Module, ModuleCodeGenerationContext,
  ModuleGraph, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType, ValueCacheVersions,
};

#[cacheable]
#[derive(Debug)]
pub struct TempModule {
  id: ModuleIdentifier,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  dependencies_block: DependenciesBlockData,
}

impl TempModule {
  pub fn transform_from(module: OwnedOrRef<BoxModule>) -> OwnedOrRef<BoxModule> {
    let m = module.as_ref();
    OwnedOrRef::Owned(BoxModule::new(Box::new(Self {
      id: m.identifier(),
      build_info: BuildInfo {
        dependencies: m.build_info().dependencies.clone(),
        ..Default::default()
      },
      build_meta: m.build_meta().clone(),
      dependencies_block: DependenciesBlockData::new(
        m.get_dependency_refs()
          .iter()
          .map(|dependency| super::TempDependency::transform_from(dependency.into()).into_owned())
          .collect(),
        Vec::new(),
      ),
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

  fn need_build_for_incremental(&self, _value_cache_versions: &ValueCacheVersions) -> bool {
    // return true to make sure this module always rebuild
    true
  }

  async fn code_generation(
    &self,
    _code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResultBuilder> {
    unreachable!()
  }

  async fn get_runtime_hash(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    unreachable!()
  }

  async fn build(
    self: Box<Self>,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BoxModule> {
    Ok(BoxModule::new(self))
  }
}

impl Identifiable for TempModule {
  fn identifier(&self) -> rspack_collections::Identifier {
    self.id
  }
}

impl DependenciesBlock for TempModule {
  fn dependencies_block(&self) -> &DependenciesBlockData {
    &self.dependencies_block
  }

  fn dependencies_block_mut(&mut self) -> &mut DependenciesBlockData {
    &mut self.dependencies_block
  }
}
