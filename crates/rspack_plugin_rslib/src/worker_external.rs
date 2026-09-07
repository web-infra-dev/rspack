use std::sync::Arc;

use rspack_core::{
  BuildModuleGraphArtifact, DependenciesBlock, Dependency, DependencyCodeGeneration,
  DependencyTemplate, ExternalModule, TemplateContext, TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::WorkerDependency;

fn should_cutout_worker_external(
  cutout_all_externals: bool,
  output_module: bool,
  external_module: &ExternalModule,
) -> bool {
  if !output_module {
    return false;
  }

  let request = &external_module.get_request().primary;
  if !request.starts_with("./") && !request.starts_with("../") {
    return false;
  }

  cutout_all_externals || matches!(external_module.resolve_external_type(), "import" | "module")
}

pub fn cutout_worker_externals(
  cutout_all_externals: bool,
  output_module: bool,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
) {
  let mg = build_module_graph_artifact.get_module_graph();
  let mut connections_to_disable = Vec::new();

  for (_, module) in mg.modules() {
    for block_id in module.get_blocks() {
      let Some(block) = mg.block_by_id(block_id) else {
        continue;
      };
      for block_dep_id in block.get_dependencies() {
        let dependency = mg.dependency_by_id(block_dep_id);
        if !dependency.as_any().is::<WorkerDependency>() {
          continue;
        }

        let Some(connection) = mg.connection_by_dependency_id(block_dep_id) else {
          continue;
        };
        let Some(module) = mg.module_by_identifier(connection.module_identifier()) else {
          continue;
        };
        let Some(external_module) = module.as_external_module() else {
          continue;
        };

        if should_cutout_worker_external(cutout_all_externals, output_module, external_module) {
          connections_to_disable.push(*block_dep_id);
        }
      }
    }
  }

  let mg = build_module_graph_artifact.get_module_graph_mut();
  for dep_id in connections_to_disable {
    let connection = mg
      .connection_by_dependency_id_mut(&dep_id)
      .expect("worker external should have a module graph connection");
    connection.force_inactive();
  }
}

fn render_worker_external_module(
  worker_dependency: &WorkerDependency,
  external_module: &ExternalModule,
  source: &mut TemplateReplaceSource,
) {
  let request = rspack_util::json_stringify_str(&external_module.get_request().primary);
  worker_dependency.replace_request(source, request);
}

#[derive(Debug)]
pub struct ExternalWorkerDependencyTemplate {
  pub cutout_all_externals: bool,
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for ExternalWorkerDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let worker_dependency = dep
      .as_any()
      .downcast_ref::<WorkerDependency>()
      .expect("ExternalWorkerDependencyTemplate should be used for WorkerDependency");
    let compilation = code_generatable_context.compilation;
    let module_graph = compilation.get_module_graph();
    let external_module = module_graph
      .module_identifier_by_dependency_id(worker_dependency.id())
      .and_then(|module_id| module_graph.module_by_identifier(module_id))
      .and_then(|module| module.as_external_module());

    if let Some(external_module) = external_module
      && should_cutout_worker_external(
        self.cutout_all_externals,
        compilation.options.output.module,
        external_module,
      )
    {
      render_worker_external_module(worker_dependency, external_module, source);
      return;
    }

    if let Some(template) = &self.template {
      template.render(dep, source, code_generatable_context);
    }
  }
}
