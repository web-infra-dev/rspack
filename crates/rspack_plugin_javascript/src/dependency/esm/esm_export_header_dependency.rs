use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyId, DependencyRange,
  DependencyTemplate, DependencyType, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

// Remove `export` label.
// Before: `export const a = 1`
// After: `const a = 1`
#[derive(Debug, Clone)]
pub struct ESMExportHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  range_decl: Option<DependencyRange>,
}

impl ESMExportHeaderDependency {
  pub fn new(range: DependencyRange, range_decl: Option<DependencyRange>) -> Self {
    Self {
      range,
      range_decl,
      id: DependencyId::default(),
    }
  }
}

impl Dependency for ESMExportHeaderDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<String> {
    Some(self.range.to_string())
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportHeader
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl DependencyTemplate for ESMExportHeaderDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.range.start,
      if let Some(range) = &self.range_decl {
        range.start
      } else {
        self.range.end
      },
      "",
      None,
    );
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

impl AsModuleDependency for ESMExportHeaderDependency {}
impl AsContextDependency for ESMExportHeaderDependency {}
