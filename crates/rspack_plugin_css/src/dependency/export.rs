use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec,
  ExportsSpec, TemplateContext, TemplateReplaceSource,
};

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

  fn get_exports(&self, _mg: &rspack_core::ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(
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
}

impl DependencyTemplate for CssExportDependency {
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

impl AsContextDependency for CssExportDependency {}
impl AsModuleDependency for CssExportDependency {}
