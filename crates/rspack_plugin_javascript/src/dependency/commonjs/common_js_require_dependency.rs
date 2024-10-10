use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{module_id, Compilation, RealDependencyLocation, RuntimeSpec};
use rspack_core::{AsContextDependency, Dependency, DependencyCategory};
use rspack_core::{DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ModuleDependency};
use rspack_core::{TemplateContext, TemplateReplaceSource};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: RealDependencyLocation,
  range_expr: Option<RealDependencyLocation>,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: String,
    range: RealDependencyLocation,
    range_expr: Option<RealDependencyLocation>,
    optional: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      range_expr,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<String> {
    Some(self.range.to_string())
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    self.range_expr.as_ref()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

#[cacheable_dyn]
impl DependencyTemplate for CommonJsRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.range.start,
      self.range.end - 1,
      module_id(
        code_generatable_context.compilation,
        &self.id,
        &self.request,
        false,
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

impl AsContextDependency for CommonJsRequireDependency {}
