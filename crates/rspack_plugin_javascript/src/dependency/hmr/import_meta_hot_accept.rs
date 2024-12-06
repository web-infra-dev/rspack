use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  module_id, AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyId,
  DependencyRange, DependencyTemplate, DependencyType, ModuleDependency, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaHotAcceptDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
}

impl ImportMetaHotAcceptDependency {
  pub fn new(request: Atom, range: DependencyRange) -> Self {
    Self {
      request,
      range,
      id: DependencyId::new(),
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
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

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn weak(&self) -> bool {
    true
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ImportMetaHotAcceptDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.range.start,
      self.range.end,
      module_id(
        code_generatable_context.compilation,
        &self.id,
        &self.request,
        self.weak(),
      )
      .as_str(),
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

impl AsContextDependency for ImportMetaHotAcceptDependency {}
