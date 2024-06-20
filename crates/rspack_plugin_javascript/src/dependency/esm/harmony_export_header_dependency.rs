use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyId, DependencyLocation,
  DependencyTemplate, DependencyType, TemplateContext, TemplateReplaceSource,
};

// Remove `export` label.
// Before: `export const a = 1`
// After: `const a = 1`
#[derive(Debug, Clone)]
pub struct HarmonyExportHeaderDependency {
  pub range: Option<DependencyLocation>,
  pub range_stmt: DependencyLocation,
  pub id: DependencyId,
}

impl HarmonyExportHeaderDependency {
  pub fn new(range: Option<DependencyLocation>, range_stmt: DependencyLocation) -> Self {
    Self {
      range,
      range_stmt,
      id: DependencyId::default(),
    }
  }
}

impl Dependency for HarmonyExportHeaderDependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportHeader
  }
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }
}

impl DependencyTemplate for HarmonyExportHeaderDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.range_stmt.start(),
      if let Some(range) = self.range.clone() {
        range.start()
      } else {
        self.range_stmt.end()
      },
      "",
      None,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsModuleDependency for HarmonyExportHeaderDependency {}
impl AsContextDependency for HarmonyExportHeaderDependency {}
