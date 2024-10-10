use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::DependencyId;
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, RealDependencyLocation,
  RuntimeSpec,
};
use rspack_core::{DependencyTemplate, RuntimeGlobals, TemplateContext};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: RealDependencyLocation,
}

impl RequireHeaderDependency {
  pub fn new(range: RealDependencyLocation) -> Self {
    Self {
      id: DependencyId::new(),
      range,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<String> {
    Some(self.range.to_string())
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for RequireHeaderDependency {}
impl AsContextDependency for RequireHeaderDependency {}

#[cacheable_dyn]
impl DependencyTemplate for RequireHeaderDependency {
  fn apply(
    &self,
    source: &mut rspack_core::TemplateReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    let TemplateContext {
      runtime_requirements,
      ..
    } = code_generatable_context;
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    source.replace(
      self.range.start,
      self.range.end,
      RuntimeGlobals::REQUIRE.name(),
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
