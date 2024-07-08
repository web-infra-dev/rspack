use rspack_core::{
  import_statement, runtime_condition_expression, AsDependency, DependencyId, DependencyTemplate,
  RuntimeCondition, TemplateContext, TemplateReplaceSource,
};

use crate::dependency::get_import_emitted_runtime;

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
    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let mut content = String::default();
    let module_graph = compilation.get_module_graph();
    self.dependency_ids.iter().for_each(|id| {
      let dependency = module_graph.dependency_by_id(id);
      let runtime_condition =
        match dependency.and_then(|dep| module_graph.get_module_by_dependency_id(dep.id())) {
          Some(ref_module) => {
            get_import_emitted_runtime(&module.identifier(), &ref_module.identifier())
          }
          None => RuntimeCondition::Boolean(false),
        };

      if matches!(runtime_condition, RuntimeCondition::Boolean(false)) {
        return;
      }

      let condition = {
        runtime_condition_expression(
          &compilation.chunk_graph,
          Some(&runtime_condition),
          *runtime,
          runtime_requirements,
        )
      };

      let request = if let Some(dependency) = dependency.and_then(|d| d.as_module_dependency()) {
        Some(dependency.request())
      } else {
        dependency
          .and_then(|d| d.as_context_dependency())
          .map(|d| d.request())
      };
      if let Some(request) = request {
        let stmts = import_statement(
          *module,
          compilation,
          runtime_requirements,
          id,
          request,
          true,
        );
        if condition == "true" {
          content.push_str(stmts.0.as_str());
          content.push_str(stmts.1.as_str());
        } else {
          content.push_str(format!("if ({}) {{\n", condition).as_str());
          content.push_str(stmts.0.as_str());
          content.push_str(stmts.1.as_str());
          content.push_str("\n}\n");
        }
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
