use rspack_core::{
  import_statement, DependencyId, DependencyTemplate, ModuleDependency, TemplateContext,
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
      if let Some(dependency) = compilation.module_graph.dependency_by_id(id) {
        let stmts = import_statement(code_generatable_context, id, dependency.request(), true);
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
}
