use rspack_core::{
  ChunkGroupOptions, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ExportsReferencedType, ModuleDependency, ModuleGraph, RuntimeGlobals,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct WorkerDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  span: Option<ErrorSpan>,
  expression: String,
  // TODO: runtime_requirements
}

impl WorkerDependency {
  pub fn new(start: u32, end: u32, expression: String, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      id: DependencyId::new(),
      expression,
      span,
    }
  }
}

impl Dependency for WorkerDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Const
  }
}

impl ModuleDependency for WorkerDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &str {
    ""
  }

  fn user_request(&self) -> &str {
    ""
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {}

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: &RuntimeSpec,
  ) -> ExportsReferencedType {
    ExportsReferencedType::No
  }
}

impl DependencyTemplate for WorkerDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    // TODO: insert runtime_requirements
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;

    source.replace(self.start, self.end, self.expression.as_str(), None);
  }
}
