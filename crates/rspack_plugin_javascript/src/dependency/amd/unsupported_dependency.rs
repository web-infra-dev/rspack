use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyId, DependencyTemplate, DependencyType, DynamicDependencyTemplate,
  DynamicDependencyTemplateType, TemplateContext, TemplateReplaceSource,
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
  fn dynamic_dependency_template(&self) -> Option<DynamicDependencyTemplateType> {
    Some(UnsupportedDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for UnsupportedDependency {}

impl AsContextDependency for UnsupportedDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct UnsupportedDependencyTemplate;

impl UnsupportedDependencyTemplate {
  pub fn template_type() -> DynamicDependencyTemplateType {
    DynamicDependencyTemplateType::CustomType("UnsupportedDependency")
  }
}

impl DynamicDependencyTemplate for UnsupportedDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyTemplate,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<UnsupportedDependency>()
      .expect("UnsupportedDependencyTemplate should only be used for UnsupportedDependency");

    let content = format!(
      "Object(function webpackMissingModule() {{var e = new Error(\"Cannot find module '{}'\"); e.code = 'MODULE_NOT_FOUND'; throw e;}}())",
      dep.request
    );
    source.replace(dep.range.0, dep.range.1, &content, None);
  }
}
