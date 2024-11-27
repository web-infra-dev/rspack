use itertools::Itertools;
use rspack_core::{
  module_raw, AffectType, AsContextDependency, AsModuleDependency, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ModuleDependency,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::atom::Atom;

use super::{
  amd_require_item_dependency::AMDRequireItemDependency,
  local_module_dependency::LocalModuleDependency,
};

#[derive(Debug, Clone)]
pub enum AmdDep {
  String(Atom),
  LocalModuleDependency(LocalModuleDependency),
  AMDRequireItemDependency(AMDRequireItemDependency),
}

#[derive(Debug, Clone)]
pub struct AmdRequireArrayDependency {
  id: DependencyId,
  deps_array: Vec<AmdDep>,
  range: (u32, u32),
}

impl AmdRequireArrayDependency {
  pub fn new(deps_array: Vec<AmdDep>, range: (u32, u32)) -> Self {
    Self {
      id: DependencyId::new(),
      deps_array,
      range,
    }
  }
}

impl Dependency for AmdRequireArrayDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Amd
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::AmdRequireArray
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AmdRequireArrayDependency {
  fn get_content(&self, code_generatable_context: &mut TemplateContext) -> String {
    format!(
      "[{}]",
      self
        .deps_array
        .iter()
        .map(|dep| Self::content_for_dependency(dep, code_generatable_context))
        .join(", ")
    )
  }

  fn content_for_dependency(
    dep: &AmdDep,
    code_generatable_context: &mut TemplateContext,
  ) -> String {
    match dep {
      AmdDep::String(name) => name.to_string(),
      AmdDep::LocalModuleDependency(dep) => dep.get_variable_name(),
      AmdDep::AMDRequireItemDependency(dep) => module_raw(
        code_generatable_context.compilation,
        code_generatable_context.runtime_requirements,
        dep.id(),
        dep.request(),
        dep.weak(),
      ),
    }
  }
}

impl DependencyTemplate for AmdRequireArrayDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let content = self.get_content(code_generatable_context);
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

impl AsModuleDependency for AmdRequireArrayDependency {}

impl AsContextDependency for AmdRequireArrayDependency {}
