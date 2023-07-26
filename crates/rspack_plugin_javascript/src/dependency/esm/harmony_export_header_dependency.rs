use rspack_core::{DependencyTemplate, TemplateContext, TemplateReplaceSource};

// Remove `export` label.
// Before: `export const a = 1`
// After: `const a = 1`
#[derive(Debug)]
pub struct HarmonyExportHeaderDependency {
  pub position: u32,
}

impl HarmonyExportHeaderDependency {
  pub fn new(position: u32) -> Self {
    Self { position }
  }
}

impl DependencyTemplate for HarmonyExportHeaderDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(self.position, self.position + 6 /* export */, "", None);
  }
}
