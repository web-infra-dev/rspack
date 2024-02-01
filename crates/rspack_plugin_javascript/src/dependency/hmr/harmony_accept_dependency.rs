use rspack_core::{
  import_statement, AsDependency, DependencyId, DependencyTemplate, TemplateContext,
  TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct HarmonyAcceptDependency {
  start: u32,
  end: u32,
  has_callback: bool,
  dependency_ids: Vec<DependencyId>,
}

impl HarmonyAcceptDependency {
  pub fn new(start: u32, end: u32, has_callback: bool, dependency_ids: Vec<DependencyId>) -> Self {
    Self {
      start,
      end,
      has_callback,
      dependency_ids,
    }
  }
}

impl DependencyTemplate for HarmonyAcceptDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;

    let mut content = String::default();

    self.dependency_ids.iter().for_each(|id| {
      let dependency = compilation.module_graph.dependency_by_id(id);
      let request = if let Some(dependency) = dependency.and_then(|d| d.as_module_dependency()) {
        Some(dependency.request())
      } else {
        dependency
          .and_then(|d| d.as_context_dependency())
          .map(|d| d.request())
      };
      if let Some(request) = request {
        let stmts = import_statement(code_generatable_context, id, request, true);
        content.push_str(stmts.0.as_str());
        content.push_str(stmts.1.as_str());
      }
    });

    if self.has_callback {
      source.insert(
        self.start,
        format!("function(__WEBPACK_OUTDATED_DEPENDENCIES__) {{\n{content}(").as_str(),
        None,
      );
      source.insert(
        self.end,
        ")(__WEBPACK_OUTDATED_DEPENDENCIES__); }.bind(this)",
        None,
      );
    } else {
      source.insert(
        self.start,
        format!(", function(){{\n{content}\n}}").as_str(),
        None,
      );
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    None
  }
}

impl AsDependency for HarmonyAcceptDependency {}
