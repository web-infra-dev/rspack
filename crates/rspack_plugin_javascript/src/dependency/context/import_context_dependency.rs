use rspack_core::{
  module_id_expr, normalize_context, CodeGeneratable, CodeGeneratableContext,
  CodeGeneratableResult, CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, ContextOptions, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, ModuleDependency, RuntimeGlobals,
};
use rspack_error::Result;

#[derive(Debug, Clone)]
pub struct ImportContextDependency {
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  pub id: Option<DependencyId>,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
}

impl ImportContextDependency {
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
      id: None,
    }
  }
}

impl Dependency for ImportContextDependency {
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
    &DependencyType::ImportContext
  }
}

impl ModuleDependency for ImportContextDependency {
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

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }
}

impl CodeGeneratable for ImportContextDependency {
  fn generate(&self, _context: &mut CodeGeneratableContext) -> Result<CodeGeneratableResult> {
    todo!()
  }
}

impl CodeReplaceSourceDependency for ImportContextDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let CodeReplaceSourceDependencyContext { compilation, .. } = code_generatable_context;

    let id: DependencyId = self.id().expect("should have dependency id");

    let module_id = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&id)
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
      source.insert(
        self.args_end,
        format!(".replace('{context}', './'))").as_str(),
        None,
      );
    }
  }
}
