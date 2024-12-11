use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  import_statement, runtime_condition_expression, AsDependency, Compilation, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, RuntimeCondition, RuntimeSpec,
  SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

use crate::dependency::import_emitted_runtime;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMAcceptDependency {
  range: DependencyRange,
  has_callback: bool,
  dependency_ids: Vec<DependencyId>,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
}

impl ESMAcceptDependency {
  pub fn new(
    range: DependencyRange,
    has_callback: bool,
    dependency_ids: Vec<DependencyId>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      range,
      has_callback,
      dependency_ids,
      source_map,
    }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ESMAcceptDependency {
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
            import_emitted_runtime::get_runtime(&module.identifier(), &ref_module.identifier())
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
        self.range.start,
        format!("function(__WEBPACK_OUTDATED_DEPENDENCIES__) {{\n{content}(").as_str(),
        None,
      );
      source.insert(
        self.range.end,
        ")(__WEBPACK_OUTDATED_DEPENDENCIES__); }.bind(this)",
        None,
      );
    } else {
      source.insert(
        self.range.start,
        format!(", function(){{\n{content}\n}}").as_str(),
        None,
      );
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    None
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsDependency for ESMAcceptDependency {}
