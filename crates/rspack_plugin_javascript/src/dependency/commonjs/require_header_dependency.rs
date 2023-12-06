use rspack_core::{AsContextDependency, AsModuleDependency, Dependency};
use rspack_core::{DependencyId, DependencyLocation};
use rspack_core::{DependencyTemplate, RuntimeGlobals, TemplateContext};

#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  loc: DependencyLocation,
}

impl RequireHeaderDependency {
  pub fn new(start: u32, end: u32) -> Self {
    let loc = DependencyLocation::new(start, end);
    Self {
      id: DependencyId::new(),
      loc,
    }
  }
}

impl Dependency for RequireHeaderDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "RequireHeaderDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
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
      self.loc.start(),
      self.loc.end() - 1,
      RuntimeGlobals::REQUIRE.name(),
      None,
    );
  }
}
