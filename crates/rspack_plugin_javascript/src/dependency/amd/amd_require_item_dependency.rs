use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AffectType, AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  FactorizeInfo, ModuleDependency, TemplateContext, TemplateReplaceSource,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct AMDRequireItemDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: Option<DependencyRange>,
  optional: bool,
  factorize_info: FactorizeInfo,
}

impl AMDRequireItemDependency {
  pub fn new(request: Atom, range: Option<DependencyRange>) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      optional: false,
      factorize_info: Default::default(),
    }
  }

  pub fn set_optional(&mut self, optional: bool) {
    self.optional = optional;
  }
}

#[cacheable_dyn]
impl Dependency for AMDRequireItemDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<DependencyRange> {
    self.range
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Amd
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::AmdRequireItem
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for AMDRequireItemDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for AMDRequireItemDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for AMDRequireItemDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(AMDRequireItemDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct AMDRequireItemDependencyTemplate;

impl AMDRequireItemDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::AmdRequireItem)
  }
}

impl DependencyTemplate for AMDRequireItemDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<AMDRequireItemDependency>()
      .expect("AMDRequireItemDependencyTemplate should only be used for AMDRequireItemDependency");

    let Some(range) = &dep.range else {
      return;
    };
    // ModuleDependencyTemplateAsRequireId
    let content = code_generatable_context.runtime_template.module_raw(
      code_generatable_context.compilation,
      &dep.id,
      &dep.request,
      dep.weak(),
    );
    source.replace(range.start, range.end, &content, None);
  }
}
