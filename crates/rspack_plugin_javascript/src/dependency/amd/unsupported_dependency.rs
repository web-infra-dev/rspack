use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, TemplateContext, TemplateReplaceSource,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct UnsupportedDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
}

impl UnsupportedDependency {
  pub fn new(request: Atom, range: DependencyRange) -> Self {
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

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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
impl DependencyCodeGeneration for UnsupportedDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(UnsupportedDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for UnsupportedDependency {}

impl AsContextDependency for UnsupportedDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct UnsupportedDependencyTemplate;

impl UnsupportedDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("UnsupportedDependency")
  }
}

impl DependencyTemplate for UnsupportedDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
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
    source.replace(dep.range.start, dep.range.end, &content, None);
  }
}
