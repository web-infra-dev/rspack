use rspack_core::{
  create_resource_identifier_for_context_dependency, module_id_expr, ContextOptions, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ModuleDependency, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct RequireContextDependency {
  start: u32,
  end: u32,
  pub id: DependencyId,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
  resource_identifier: String,
}

impl RequireContextDependency {
  pub fn new(start: u32, end: u32, options: ContextOptions, span: Option<ErrorSpan>) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(&options);
    Self {
      start,
      end,
      options,
      span,
      id: DependencyId::new(),
      resource_identifier: resource_identifier,
    }
  }
}

impl Dependency for RequireContextDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireContext
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for RequireContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

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

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {
    self.options.request = request;
  }
}

impl DependencyTemplate for RequireContextDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;

    let module_id = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&self.id)
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
