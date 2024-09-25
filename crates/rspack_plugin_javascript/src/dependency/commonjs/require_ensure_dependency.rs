use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ModuleDependency, RealDependencyLocation,
};

#[derive(Debug, Clone)]
pub struct RequireEnsureDependency {
  id: DependencyId,
  range: RealDependencyLocation,
  content_range: RealDependencyLocation,
  error_handler_range: Option<RealDependencyLocation>,
}

impl RequireEnsureDependency {
  pub fn new(
    range: RealDependencyLocation,
    content_range: RealDependencyLocation,
    error_handler_range: Option<RealDependencyLocation>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      content_range,
      error_handler_range,
    }
  }
}

impl Dependency for RequireEnsureDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &rspack_core::DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireEnsure
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    todo!()
  }
}

impl ModuleDependency for RequireEnsureDependency {
  fn request(&self) -> &str {
    todo!()
  }
}

impl DependencyTemplate for RequireEnsureDependency {
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

impl AsContextDependency for RequireEnsureDependency {}
