use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyId, DependencyTemplate, DependencyType, TemplateContext, TemplateReplaceSource,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct UnsupportedDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
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

#[cacheable_dyn]
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

#[cacheable_dyn]
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
}

impl AsModuleDependency for UnsupportedDependency {}

impl AsContextDependency for UnsupportedDependency {}
