use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec,
  ExportsSpec, TemplateContext, TemplateReplaceSource,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CssLocalIdentDependency {
  id: DependencyId,
  name: String,
  local_ident: String,
  start: u32,
  end: u32,
}

impl CssLocalIdentDependency {
  pub fn new(name: String, local_ident: String, start: u32, end: u32) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      local_ident,
      start,
      end,
    }
  }
}

impl Dependency for CssLocalIdentDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "CssLocalIdentDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssLocalIdent
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssLocalIdent
  }

  fn get_exports(&self, _mg: &rspack_core::ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
        name: self.name.as_str().into(),
        can_mangle: Some(false),
        ..Default::default()
      })]),
      ..Default::default()
    })
  }
}

impl DependencyTemplate for CssLocalIdentDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(self.start, self.end, &self.local_ident, None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for CssLocalIdentDependency {}
impl AsModuleDependency for CssLocalIdentDependency {}
