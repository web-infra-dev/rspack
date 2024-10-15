use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyId,
  DependencyTemplate, RealDependencyLocation, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireResolveHeaderDependency {
  id: DependencyId,
  range: RealDependencyLocation,
}

impl RequireResolveHeaderDependency {
  pub fn new(range: RealDependencyLocation) -> Self {
    Self {
      id: DependencyId::new(),
      range,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireResolveHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<String> {
    Some(self.range.to_string())
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for RequireResolveHeaderDependency {}
impl AsContextDependency for RequireResolveHeaderDependency {}

#[cacheable_dyn]
impl DependencyTemplate for RequireResolveHeaderDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.range.start,
      self.range.end,
      "/*require.resolve*/",
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
