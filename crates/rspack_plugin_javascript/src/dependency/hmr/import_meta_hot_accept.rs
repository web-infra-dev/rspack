use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, FactorizeInfo,
  ModuleDependency, TemplateContext, TemplateReplaceSource, module_id,
};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaHotAcceptDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  factorize_info: FactorizeInfo,
}

impl ImportMetaHotAcceptDependency {
  pub fn new(request: Atom, range: DependencyRange) -> Self {
    Self {
      request,
      range,
      id: DependencyId::new(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ImportMetaHotAcceptDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaHotAccept
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ImportMetaHotAcceptDependency {
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
impl DependencyCodeGeneration for ImportMetaHotAcceptDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaHotAcceptDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ImportMetaHotAcceptDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaHotAcceptDependencyTemplate;

impl ImportMetaHotAcceptDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportMetaHotAccept)
  }
}

impl DependencyTemplate for ImportMetaHotAcceptDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaHotAcceptDependency>()
      .expect(
        "ImportMetaHotAcceptDependencyTemplate should be used for ImportMetaHotAcceptDependency",
      );

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
