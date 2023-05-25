use rspack_core::{
  module_id_expr, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct HarmonyAcceptDependency {
  start: u32,
  end: u32,
  has_callback: bool,
  id: Option<DependencyId>,
  request: JsWord,
  span: Option<ErrorSpan>,
}

impl HarmonyAcceptDependency {
  pub fn new(
    start: u32,
    end: u32,
    has_callback: bool,
    request: JsWord,
    span: Option<ErrorSpan>,
  ) -> Self {
    Self {
      start,
      end,
      has_callback,
      request,
      span,
      id: None,
    }
  }
}

impl Dependency for HarmonyAcceptDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaHotAccept
  }
}

impl ModuleDependency for HarmonyAcceptDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }
}

impl CodeGeneratable for HarmonyAcceptDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}

impl CodeReplaceSourceDependency for HarmonyAcceptDependency {
  fn apply(
    &self,
    _source: &mut CodeReplaceSourceDependencyReplaceSource,
    _code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    todo!()
  }
}
