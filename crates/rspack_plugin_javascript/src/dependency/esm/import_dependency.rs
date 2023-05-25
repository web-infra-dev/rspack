use rspack_core::{
  module_id_expr, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency, RuntimeGlobals,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ImportDependency {
  start: u32,
  end: u32,
  id: Option<DependencyId>,
  request: JsWord,
  span: Option<ErrorSpan>,
  /// Used to store `webpackChunkName` in `import("/* webpackChunkName: "plugins/myModule" */")`
  pub name: Option<String>,
}

impl ImportDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: JsWord,
    span: Option<ErrorSpan>,
    name: Option<String>,
  ) -> Self {
    Self {
      start,
      end,
      request,
      span,
      id: None,
      name,
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

    let CodeReplaceSourceDependencyContext {
      runtime_requirements,
      compilation,
      ..
    } = code_generatable_context;

    let module_id = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&id)
      .map(|m| m.id(&compilation.chunk_graph))
      .expect("should have dependency id");

    let module_id_str = module_id_expr(&self.request, module_id);

    // Add interop require to runtime requirements, as dynamic imports have been transformed so `inject_runtime_helper` will not be able to detect this.
    runtime_requirements.insert(RuntimeGlobals::INTEROP_REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::LOAD_CHUNK_WITH_MODULE);

    source.replace(
      self.start,
      self.end,
      format!(
        "__webpack_require__.el({module_id_str}).then(__webpack_require__.bind(__webpack_require__, {module_id_str})).then(__webpack_require__.ir)",
      )
      .as_str(),
      None,
    );
  }
}
