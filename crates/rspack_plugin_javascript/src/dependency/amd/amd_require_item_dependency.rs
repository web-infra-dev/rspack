use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  module_raw, AffectType, AsContextDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, FactorizeInfo, ModuleDependency, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct AMDRequireItemDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: Option<(u32, u32)>,
  optional: bool,
  factorize_info: FactorizeInfo,
}

impl AMDRequireItemDependency {
  pub fn new(request: Atom, range: Option<(u32, u32)>) -> Self {
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
impl DependencyTemplate for AMDRequireItemDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let Some(range) = &self.range else {
      return;
    };
    // ModuleDependencyTemplateAsRequireId
    let content = module_raw(
      code_generatable_context.compilation,
      code_generatable_context.runtime_requirements,
      &self.id,
      &self.request,
      self.weak(),
    );
    source.replace(range.0, range.1, &content, None);
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
