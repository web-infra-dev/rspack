use rspack_core::{
  module_raw, AffectType, AsContextDependency, Compilation, Dependency, DependencyCategory,
  DependencyId, DependencyTemplate, DependencyType, ModuleDependency, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::atom::Atom;

#[derive(Debug, Clone)]
pub struct AMDRequireItemDependency {
  id: DependencyId,
  request: Atom,
  range: (u32, u32),
  optional: bool,
}

impl AMDRequireItemDependency {
  pub fn new(request: Atom, range: (u32, u32)) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      optional: false,
    }
  }

  pub fn get_optional(&self) -> bool {
    self.optional
  }

  pub fn set_optional(&mut self, optional: bool) {
    self.optional = optional;
  }
}

impl Dependency for AMDRequireItemDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Amd
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::AmdRequireItem
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl DependencyTemplate for AMDRequireItemDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    // ModuleDependencyTemplateAsRequireId
    let content = module_raw(
      code_generatable_context.compilation,
      code_generatable_context.runtime_requirements,
      &self.id,
      &self.request,
      self.weak(),
    );
    source.replace(self.range.0, self.range.1, &content, None);
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

impl ModuleDependency for AMDRequireItemDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsContextDependency for AMDRequireItemDependency {}
