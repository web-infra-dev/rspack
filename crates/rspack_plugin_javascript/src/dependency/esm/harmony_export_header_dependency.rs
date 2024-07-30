use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, TemplateContext, TemplateReplaceSource,
};
use rspack_error::ErrorLocation;

// Remove `export` label.
// Before: `export const a = 1`
// After: `const a = 1`
#[derive(Debug, Clone)]
pub struct HarmonyExportHeaderDependency {
  id: DependencyId,
  loc: ErrorLocation,
  range: Option<ErrorSpan>,
  range_stmt: ErrorSpan,
}

impl HarmonyExportHeaderDependency {
  pub fn new(loc: ErrorLocation, range: Option<ErrorSpan>, range_stmt: ErrorSpan) -> Self {
    Self {
      loc,
      range,
      range_stmt,
      id: DependencyId::default(),
    }
  }
}

impl Dependency for HarmonyExportHeaderDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<ErrorLocation> {
    Some(self.loc)
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportHeader
  }
}

impl DependencyTemplate for HarmonyExportHeaderDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.range_stmt.start,
      if let Some(range) = self.range.clone() {
        range.start
      } else {
        self.range_stmt.end
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
