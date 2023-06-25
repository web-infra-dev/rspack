use rspack_core::{
  module_id_expr, CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource,
  ContextOptions, Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan,
  ModuleDependency, RuntimeGlobals,
};

#[derive(Debug, Clone)]
pub struct RequireContextDependency {
  start: u32,
  end: u32,
  pub id: Option<DependencyId>,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
}

impl RequireContextDependency {
  pub fn new(start: u32, end: u32, options: ContextOptions, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      options,
      span,
      id: None,
    }
  }
}

impl Dependency for RequireContextDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireContext
  }
}

impl ModuleDependency for RequireContextDependency {
  fn request(&self) -> &str {
    &self.options.request
  }

  fn user_request(&self) -> &str {
    &self.options.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn options(&self) -> Option<&ContextOptions> {
    Some(&self.options)
  }

  fn as_code_generatable_dependency(&self) -> Option<Box<&dyn CodeGeneratableDependency>> {
    Some(Box::new(self))
  }

  fn set_request(&mut self, request: String) {
    self.options.request = request;
  }
}

impl CodeGeneratableDependency for RequireContextDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;

    let id: DependencyId = self.id().expect("should have dependency id");

    let module_id = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&id)
      .map(|m| m.id(&compilation.chunk_graph))
      .expect("should have dependency id");

    let module_id_str = module_id_expr(&self.options.request, module_id);

    source.replace(
      self.start,
      self.end,
      format!("{}({module_id_str})", RuntimeGlobals::REQUIRE,).as_str(),
      None,
    );
  }
}
