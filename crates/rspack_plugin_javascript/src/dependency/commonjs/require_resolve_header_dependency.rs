use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, RuntimeSpec, SharedSourceMap,
  TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct RequireResolveHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  source_map: Option<SharedSourceMap>,
}

impl RequireResolveHeaderDependency {
  pub fn new(range: DependencyRange, source_map: Option<SharedSourceMap>) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      source_map,
    }
  }
}

impl Dependency for RequireResolveHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    Some(self.range.to_loc(self.source_map.as_ref()))
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for RequireResolveHeaderDependency {}
impl AsContextDependency for RequireResolveHeaderDependency {}

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
