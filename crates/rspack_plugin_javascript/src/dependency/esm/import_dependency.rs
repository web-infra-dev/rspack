use rspack_core::{
  module_namespace_promise, ChunkGroupOptions, CodeGeneratable, CodeGeneratableContext,
  CodeGeneratableResult, CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ImportDependency {
  start: u32,
  end: u32,
  id: Option<DependencyId>,
  request: JsWord,
  span: Option<ErrorSpan>,
  /// This is used to implement `webpackChunkName`, `webpackPrefetch` etc.
  /// for example: `import(/* webpackChunkName: "my-chunk-name", webpackPrefetch: true */ './module')`
  pub group_options: ChunkGroupOptions,
}

impl ImportDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: JsWord,
    span: Option<ErrorSpan>,
    group_options: ChunkGroupOptions,
  ) -> Self {
    Self {
      start,
      end,
      request,
      span,
      id: None,
      group_options,
    }
  }
}

impl Dependency for ImportDependency {
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
    &DependencyType::DynamicImport
  }
}

impl ModuleDependency for ImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }

  fn group_options(&self) -> Option<&ChunkGroupOptions> {
    Some(&self.group_options)
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl CodeGeneratable for ImportDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}

impl CodeReplaceSourceDependency for ImportDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let id: DependencyId = self.id().expect("should have dependency id");
    source.replace(
      self.start,
      self.end,
      module_namespace_promise(code_generatable_context, &id, &self.request, false).as_str(),
      None,
    );
  }
}
