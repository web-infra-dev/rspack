use rspack_core::{
  module_id_expr, normalize_context, CodeGeneratableContext, CodeGeneratableDependency,
  CodeGeneratableSource, ContextOptions, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency, RuntimeGlobals,
};

#[derive(Debug, Clone)]
pub struct CommonJsRequireContextDependency {
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  pub id: DependencyId,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
}

impl CommonJsRequireContextDependency {
  pub fn new(
    callee_start: u32,
    callee_end: u32,
    args_end: u32,
    options: ContextOptions,
    span: Option<ErrorSpan>,
  ) -> Self {
    Self {
      callee_start,
      callee_end,
      args_end,
      options,
      span,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for CommonJsRequireContextDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CommonJSRequireContext
  }
}

impl ModuleDependency for CommonJsRequireContextDependency {
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

  fn as_code_generatable_dependency(&self) -> Option<&dyn CodeGeneratableDependency> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {
    self.options.request = request;
  }
}

impl CodeGeneratableDependency for CommonJsRequireContextDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;

    let module_id = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&self.id)
      .map(|m| m.id(&compilation.chunk_graph))
      .expect("should have dependency id");

    let module_id_str = module_id_expr(&self.options.request, module_id);

    source.replace(
      self.callee_start,
      self.callee_end,
      format!("{}({module_id_str})", RuntimeGlobals::REQUIRE,).as_str(),
      None,
    );

    let context = normalize_context(&self.options.request);

    if !context.is_empty() {
      source.insert(self.callee_end, "(", None);
      source.insert(
        self.args_end,
        format!(".replace('{context}', './'))").as_str(),
        None,
      );
    }
  }
}
