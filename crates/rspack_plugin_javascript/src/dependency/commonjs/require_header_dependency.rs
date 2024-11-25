use rspack_core::DependencyId;
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyRange, RuntimeSpec,
};
use rspack_core::{DependencyTemplate, RuntimeGlobals, TemplateContext};

#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
}

impl RequireHeaderDependency {
  pub fn new(range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      range,
    }
  }
}

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
