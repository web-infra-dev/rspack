use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, FactorizeInfo,
  ModuleDependency, TemplateContext, TemplateReplaceSource, module_id,
};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleHotAcceptDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  factorize_info: FactorizeInfo,
}

impl ModuleHotAcceptDependency {
  pub fn new(request: Atom, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ModuleHotAcceptDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ModuleHotAccept
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ModuleHotAcceptDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn weak(&self) -> bool {
    true
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ModuleHotAcceptDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ModuleHotAcceptDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ModuleHotAcceptDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ModuleHotAcceptDependencyTemplate;

impl ModuleHotAcceptDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ModuleHotAccept)
  }
}

impl DependencyTemplate for ModuleHotAcceptDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ModuleHotAcceptDependency>()
      .expect("ModuleHotAcceptDependencyTemplate should be used for ModuleHotAcceptDependency");

    source.replace(
      dep.range.start,
      dep.range.end,
      module_id(
        code_generatable_context.compilation,
        &dep.id,
        &dep.request,
        dep.weak(),
      )
      .as_str(),
      None,
    );
  }
}
