use std::{borrow::Cow, sync::Arc};

use rspack_cacheable::{cacheable, cacheable_dyn, utils::OwnedOrRef};
use rspack_collections::Identifiable;
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::RspackHashDigest;
use rspack_sources::BoxSource;
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};

use crate::{
  BoxModule, BuildContext, BuildInfo, BuildMeta, CodeGenerationResultBuilder, Compilation, Context,
  DependenciesBlock, DependenciesBlockData, FactoryMeta, FreezeLock, Module,
  ModuleCodeGenerationContext, ModuleGraph, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType,
  ValueCacheVersions,
};

#[cacheable]
#[derive(Debug)]
pub struct TempModule {
  id: ModuleIdentifier,
  build_info: FreezeLock<BuildInfo>,
  build_meta: FreezeLock<BuildMeta>,
  dependencies_block: DependenciesBlockData,
}

impl TempModule {
  pub fn transform_from(module: OwnedOrRef<crate::ModuleRef>) -> OwnedOrRef<crate::ModuleRef> {
    let m = module.as_ref();
    let module = BoxModule::new(Box::new(Self {
      id: m.identifier(),
      build_info: BuildInfo {
        dependencies: m.build_info().dependencies.clone(),
        ..Default::default()
      }
      .into(),
      build_meta: m.freeze_build_meta().clone().into(),
      dependencies_block: DependenciesBlockData::new(
        m.get_dependencies()
          .iter()
          .map(|dependency| super::TempDependency::transform_from(dependency.into()).into_owned())
          .collect(),
        Vec::new(),
      ),
    }));
    module.freeze_build_info();
    OwnedOrRef::Owned(module.into())
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
  fn factory_meta(&self) -> Option<Arc<FactoryMeta>> {
    unreachable!()
  }

  fn set_factory_meta(&self, _factory_meta: FactoryMeta) {
    unreachable!()
  }

  fn reset_for_compilation(&self, _factory_meta: Option<Arc<FactoryMeta>>) {
    unreachable!()
  }

  fn build_info(&self) -> crate::FreezeReadGuard<'_, BuildInfo> {
    self.build_info.read()
  }

  fn build_info_unchecked(&self) -> &BuildInfo {
    self.build_info.get_unchecked()
  }

  fn freeze_build_info(&self) {
    self.build_info.freeze();
  }

  fn extend_build_assets(&self, assets: crate::CompilationAssets) {
    self.build_info.extend_assets(assets);
  }

  fn build_info_mut(&mut self) -> &mut BuildInfo {
    self.build_info.get_mut()
  }

  fn build_meta(&self) -> crate::FreezeReadGuard<'_, BuildMeta> {
    self.build_meta.read()
  }

  fn build_meta_unchecked(&self) -> &BuildMeta {
    self.build_meta.get_unchecked()
  }

  fn freeze_build_meta(&self) -> &triomphe::Arc<BuildMeta> {
    self.build_meta.freeze()
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
