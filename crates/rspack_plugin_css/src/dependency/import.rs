use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyId, DependencyRange,
  DependencyTemplate, DependencyType, ModuleDependency, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssImportDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
}

impl CssImportDependency {
  pub fn new(request: String, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssImport
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

#[cacheable_dyn]
impl DependencyTemplate for CssImportDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(self.range.start, self.range.end, "", None);
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

impl AsContextDependency for CssImportDependency {}
