use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, ImportPhase, RuntimeCondition, SharedSourceMap, TemplateContext,
  TemplateReplaceSource,
};

use crate::dependency::import_emitted_runtime;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMAcceptDependency {
  range: DependencyRange,
  has_callback: bool,
  dependency_ids: Vec<DependencyId>,
  loc: Option<DependencyLocation>,
}

impl ESMAcceptDependency {
  pub fn new(
    range: DependencyRange,
    has_callback: bool,
    dependency_ids: Vec<DependencyId>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    let loc = range.to_loc(source_map.as_deref());
    Self {
      range,
      has_callback,
      dependency_ids,
      loc,
    }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMAcceptDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMAcceptDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMAcceptDependencyTemplate;

impl ESMAcceptDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ESMAcceptDependency")
  }
}

impl DependencyTemplate for ESMAcceptDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMAcceptDependency>()
      .expect("ESMAcceptDependencyTemplate should be used for ESMAcceptDependency");

    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let mut content = String::default();
    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    dep.dependency_ids.iter().for_each(|id| {
      let dependency = module_graph
        .dependency_by_id(id)
        .expect("should have dependency");
      let target_module = module_graph.get_module_by_dependency_id(dependency.id());
      let runtime_condition = match target_module {
        Some(target_module) => {
          import_emitted_runtime::get_runtime(&module_identifier, &target_module.identifier())
        }
        None => RuntimeCondition::Boolean(false),
      };

      if matches!(runtime_condition, RuntimeCondition::Boolean(false)) {
        return;
      }

      let condition = {
        compilation.runtime_template.runtime_condition_expression(
          &compilation.chunk_graph,
          Some(&runtime_condition),
          *runtime,
          runtime_requirements,
        )
      };

      let module_dependency = dependency
        .as_module_dependency()
        .expect("should be module dependency");
      let phase = ImportPhase::Evaluation;
      let import_var = compilation.get_import_var(
        module_identifier,
        target_module,
        module_dependency.user_request(),
        phase,
        *runtime,
      );
      let stmts = compilation.runtime_template.import_statement(
        *module,
        compilation,
        runtime_requirements,
        id,
        &import_var,
        module_dependency.request(),
        phase,
        true,
      );
      if condition == "true" {
        content.push_str(stmts.0.as_str());
        content.push_str(stmts.1.as_str());
      } else {
        content.push_str(format!("if ({condition}) {{\n").as_str());
        content.push_str(stmts.0.as_str());
        content.push_str(stmts.1.as_str());
        content.push_str("\n}\n");
      }
    });

    if dep.has_callback {
      source.insert(
        dep.range.start,
        format!("function(__rspack_hmr_outdated) {{\n{content}(").as_str(),
        None,
      );
      source.insert(
        dep.range.end,
        ")(__rspack_hmr_outdated); }.bind(this)",
        None,
      );
    } else {
      source.insert(
        dep.range.start,
        format!(", function(){{\n{content}\n}}").as_str(),
        None,
      );
    }
  }
}
