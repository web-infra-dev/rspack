use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ModuleDependency, RealDependencyLocation,
};
use rspack_util::atom::Atom;

#[derive(Debug, Clone)]
pub struct RequireEnsureItemDependency {
  id: DependencyId,
  request: Atom,
  range: RealDependencyLocation,
}

impl RequireEnsureItemDependency {
  pub fn new(request: Atom, range: RealDependencyLocation) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
    }
  }
}

impl Dependency for RequireEnsureItemDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &rspack_core::DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireEnsureItem
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    todo!()
  }
}

impl ModuleDependency for RequireEnsureItemDependency {
  fn request(&self) -> &str {
    todo!()
  }
}

impl DependencyTemplate for RequireEnsureItemDependency {
  fn apply(
    &self,
    source: &mut rspack_core::TemplateReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    todo!()
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &rspack_core::Compilation,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) {
    todo!()
  }
}

impl AsContextDependency for RequireEnsureItemDependency {}
