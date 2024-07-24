use std::sync::Arc;

use rspack_core::module_id;
use rspack_core::{AsContextDependency, Dependency, DependencyCategory, DependencyLocation};
use rspack_core::{DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ErrorSpan, ModuleDependency};
use rspack_core::{TemplateContext, TemplateReplaceSource};
use swc_core::common::SourceMap;

#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  loc: DependencyLocation,
  span: Option<ErrorSpan>,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: String,
    span: Option<ErrorSpan>,
    start: u32,
    end: u32,
    source: Option<Arc<SourceMap>>,
    optional: bool,
  ) -> Self {
    let loc = DependencyLocation::new(start, end, source);
    Self {
      id: DependencyId::new(),
      request,
      optional,
      loc,
      span,
    }
  }
}

impl Dependency for CommonJsRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }
}

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

impl DependencyTemplate for CommonJsRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.loc.start(),
      self.loc.end() - 1,
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
}

impl AsContextDependency for CommonJsRequireDependency {}
