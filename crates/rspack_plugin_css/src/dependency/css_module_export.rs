use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec,
  ExportsSpec, TemplateContext, TemplateReplaceSource,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CssModuleExportDependency {
  id: DependencyId,
  name: String,
  value: String,
}

impl CssModuleExportDependency {
  pub fn new(name: String, value: String) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      value,
    }
  }
}

impl Dependency for CssModuleExportDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "CssModuleExportDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssModuleExport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssModuleExport
  }

  fn get_exports(&self, _mg: &rspack_core::ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
        name: self.name.as_str().into(),
        can_mangle: Some(true),
        ..Default::default()
      })]),
      ..Default::default()
    })
  }
}

impl DependencyTemplate for CssModuleExportDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    // TODO: currently our css module implementation is different from `webpack`, so we do
    // nothing here. ref: https://github.com/webpack/webpack/blob/b9fb99c63ca433b24233e0bbc9ce336b47872c08/lib/dependencies/CssExportDependency.js#L85-L86
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for CssModuleExportDependency {}
impl AsModuleDependency for CssModuleExportDependency {}
