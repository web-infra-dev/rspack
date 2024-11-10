use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyId, DependencyTemplate, DependencyType, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::atom::Atom;

#[derive(Debug, Clone)]
pub struct UnsupportedDependency {
  id: DependencyId,
  request: Atom,
  range: (u32, u32),
}

impl UnsupportedDependency {
  pub fn new(request: Atom, range: (u32, u32)) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
    }
  }
}

impl Dependency for UnsupportedDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl DependencyTemplate for UnsupportedDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let content = format!(
      "Object(function webpackMissingModule() {{var e = new Error(\"Cannot find module '{}'\"); e.code = 'MODULE_NOT_FOUND'; throw e;}}())",
      self.request
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

impl AsModuleDependency for UnsupportedDependency {}

impl AsContextDependency for UnsupportedDependency {}
