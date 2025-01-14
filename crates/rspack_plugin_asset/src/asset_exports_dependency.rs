use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyId,
  DependencyTemplate, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
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

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::String("default".into())]),
      priority: Some(1),
      terminal_binding: Some(true),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for AssetExportsDependency {}
impl AsContextDependency for AssetExportsDependency {}

#[cacheable_dyn]
impl DependencyTemplate for AssetExportsDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}
