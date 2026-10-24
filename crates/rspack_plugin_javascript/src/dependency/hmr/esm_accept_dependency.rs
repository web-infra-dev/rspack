use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  DependencyCodeGeneration, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, ImportPhase, ModuleGraph, RuntimeCondition, TemplateContext,
  TemplateReplaceSource,
};

use crate::dependency::import_emitted_runtime;

#[cacheable]
#[derive(Debug)]
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
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      range,
      has_callback,
      dependency_ids,
      loc,
    }
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
      runtime_template,
      ..
    } = code_generatable_context;

    let mut content = String::default();
    let mut async_imports = vec![];
    let mut async_compat = String::new();
    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    dep.dependency_ids.iter().for_each(|id| {
      let dependency = module_graph.dependency_by_id(id);
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
        runtime_template.runtime_condition_expression(
          &compilation.build_chunk_graph_artifact.chunk_graph,
          Some(&runtime_condition),
          *runtime,
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
      let stmts = runtime_template.import_statement(
        *module,
        compilation,
        id,
        &import_var,
        module_dependency.request(),
        phase,
        true,
      );
      let is_async = target_module.is_some_and(|target_module| {
        ModuleGraph::is_async(
          &compilation.async_modules_artifact,
          &target_module.identifier(),
        )
      });
      let import_content = if is_async {
        async_imports.push(import_var);
        if condition == "true" {
          async_compat.push_str(&stmts.1);
        } else if !stmts.1.is_empty() {
          async_compat.push_str(&format!("if ({condition}) {{\n{}\n}}\n", stmts.1));
        }
        stmts.0
      } else {
        format!("{}{}", stmts.0, stmts.1)
      };
      if condition == "true" {
        content.push_str(&import_content);
      } else {
        content.push_str(format!("if ({condition}) {{\n").as_str());
        content.push_str(&import_content);
        content.push_str("\n}\n");
      }
    });

    let opt_async = if async_imports.is_empty() {
      ""
    } else {
      let imports = async_imports.join(", ");
      content.push_str(&format!(
        "var __rspack_async_deps = __rspack_load_async_deps([{imports}]);\n([{imports}] = __rspack_async_deps.then ? (await __rspack_async_deps)() : __rspack_async_deps);\n{async_compat}"
      ));
      "async "
    };

    if dep.has_callback {
      source.insert(
        dep.range.start,
        format!("{opt_async}function(__rspack_hmr_outdated) {{\n{content}return ("),
        None,
      );
      source.insert_static(
        dep.range.end,
        ")(__rspack_hmr_outdated); }.bind(this)",
        None,
      );
    } else {
      source.insert(
        dep.range.start,
        format!(", {opt_async}function(){{\n{content}\n}}"),
        None,
      );
    }
  }
}
