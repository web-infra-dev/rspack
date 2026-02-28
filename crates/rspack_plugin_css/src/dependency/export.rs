use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, AsModuleDependency, Dependency,
  DependencyCategory, DependencyId, DependencyType, ExportNameOrSpec, ExportSpec,
  ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssExportDependency {
  id: DependencyId,
  convention_names: Vec<String>,
}

impl CssExportDependency {
  pub fn new(convention_names: Vec<String>) -> Self {
    Self {
      id: DependencyId::new(),
      convention_names,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssExportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssExport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssExport
  }

  fn get_exports(
    &self,
    _mg: &rspack_core::ModuleGraph,
    _mg_cache: &rspack_core::ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(
        self
          .convention_names
          .iter()
          .map(|name| {
            ExportNameOrSpec::ExportSpec(ExportSpec {
              name: name.as_str().into(),
              can_mangle: Some(false),
              ..Default::default()
            })
          })
          .collect(),
      ),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsDependencyCodeGeneration for CssExportDependency {}
impl AsContextDependency for CssExportDependency {}
impl AsModuleDependency for CssExportDependency {}
