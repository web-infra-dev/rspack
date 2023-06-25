use rspack_core::{
  import_statement, CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource,
  Dependency, DependencyCategory, DependencyType, ModuleDependency, ModuleIdentifier,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct HarmonyAcceptDependency {
  start: u32,
  end: u32,
  has_callback: bool,
  module_identifier: ModuleIdentifier,
  deps: Vec<(JsWord, DependencyCategory, DependencyType)>,
}

impl HarmonyAcceptDependency {
  pub fn new(
    start: u32,
    end: u32,
    has_callback: bool,
    module_identifier: ModuleIdentifier,
    deps: Vec<(JsWord, DependencyCategory, DependencyType)>,
  ) -> Self {
    Self {
      start,
      end,
      has_callback,
      module_identifier,
      deps,
    }
  }
}

impl CodeGeneratableDependency for HarmonyAcceptDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;

    let dependencies = {
      let ids = compilation
        .module_graph
        .dependencies_by_module_identifier(&self.module_identifier)
        .expect("should have dependencies");
      ids
        .iter()
        .map(|id| {
          compilation
            .module_graph
            .dependency_by_id(id)
            .expect("should have dependency")
        })
        .collect::<Vec<_>>()
    };

    let mut content = String::default();

    self.deps.iter().for_each(|dep| {
      if let Some(dep) = dependencies
        .iter()
        .find(|d| d.request() == &dep.0 && d.category() == &dep.1 && d.dependency_type() == &dep.2)
      {
        let stmts = import_statement(
          code_generatable_context,
          &dep.id().expect("should have dependency"),
          dep.request(),
          true,
        );
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
