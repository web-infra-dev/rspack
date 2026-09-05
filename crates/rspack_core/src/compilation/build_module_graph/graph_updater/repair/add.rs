use rspack_error::Result;

use super::{TaskContext, build::BuildTask, lazy::process_unlazy_dependencies};
use crate::{
  BoxModule, DependencyRef, ModuleIdentifier,
  compilation::build_module_graph::ForwardedIdSet,
  module_graph::{ModuleGraph, ModuleGraphModule},
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: BoxModule,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<DependencyRef>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for AddTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let Self {
      original_module_identifier,
      module,
      module_graph_module,
      dependencies,
      from_unlazy,
    } = *self;
    let module_identifier = module.identifier();

    // reuse module for self referenced module
    if module.as_self_module().is_some() {
      let issuer = module_graph_module
        .issuer()
        .identifier()
        .expect("self module should have issuer");

      set_resolved_module(
        &mut context.artifact.module_graph,
        original_module_identifier,
        dependencies,
        *issuer,
      )?;

      return Ok(vec![]);
    }

    let forwarded_ids = ForwardedIdSet::from_dependencies(&dependencies);

    // reuse module if module is already added by other dependency
    if context
      .artifact
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .is_some()
    {
      set_resolved_module(
        &mut context.artifact.module_graph,
        original_module_identifier,
        dependencies,
        module_identifier,
      )?;

      if from_unlazy {
        context
          .artifact
          .affected_modules
          .mark_as_add(&module_identifier);
      }

      if context
        .artifact
        .module_graph
        .module_by_identifier(&module_identifier)
        .is_some()
      {
        if context
          .artifact
          .module_to_lazy_make
          .has_lazy_dependencies(&module_identifier)
          && !forwarded_ids.is_empty()
        {
          if let Some(task) = process_unlazy_dependencies(
            &context.artifact.module_to_lazy_make,
            &mut context.artifact.module_graph,
            forwarded_ids,
            module_identifier,
          ) {
            return Ok(vec![Box::new(task)]);
          }
          return Ok(vec![]);
        }
      } else {
        let pending_forwarded_ids = context
          .artifact
          .module_to_lazy_make
          .pending_forwarded_ids(module_identifier);
        pending_forwarded_ids.append(forwarded_ids);
      }

      return Ok(vec![]);
    }

    context
      .artifact
      .module_graph
      .add_module_graph_module(*module_graph_module);

    context
      .exports_info_artifact
      .new_exports_info(module_identifier);

    set_resolved_module(
      &mut context.artifact.module_graph,
      original_module_identifier,
      dependencies,
      module_identifier,
    )?;

    tracing::trace!("Module added: {module_identifier}");
    context
      .artifact
      .affected_modules
      .mark_as_add(&module_identifier);

    Ok(vec![Box::new(BuildTask {
      compiler_id: context.compiler_id,
      compilation_id: context.compilation_id,
      module,
      resolver_factory: context.resolver_factory.clone(),
      compiler_options: context.compiler_options.clone(),
      loader_cache: context.cache.facade("loader"),
      file_system_info: context.file_system_info.clone(),
      plugin_driver: context.plugin_driver.clone(),
      runtime_template: context.runtime_template.create_module_code_template(),
      fs: context.fs.clone(),
      forwarded_ids,
      module_build_cache: context.module_build_cache.clone(),
      value_cache_versions: context.value_cache_versions.clone(),
      use_cache: !context.rebuild_modules.contains(&module_identifier),
    })])
  }
}

fn set_resolved_module(
  module_graph: &mut ModuleGraph,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<DependencyRef>,
  module_identifier: ModuleIdentifier,
) -> Result<()> {
  for dependency in dependencies {
    module_graph.set_resolved_module(
      original_module_identifier,
      *dependency.id(),
      module_identifier,
    )?;
    module_graph.add_dependency_ref(dependency);
  }
  Ok(())
}
