use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, AsModuleDependency, Dependency, DependencyId,
  ExportNameOrSpec, ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
  ModuleGraphCacheArtifact,
};

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct AssetExportsDependency {
  id: DependencyId,
}

impl AssetExportsDependency {
  pub fn new() -> Self {
    Self {
      id: DependencyId::new(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for AssetExportsDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::String("default".into())]),
      priority: Some(1),
      terminal_binding: Some(true),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsDependencyCodeGeneration for AssetExportsDependency {}
impl AsModuleDependency for AssetExportsDependency {}
impl AsContextDependency for AssetExportsDependency {}
