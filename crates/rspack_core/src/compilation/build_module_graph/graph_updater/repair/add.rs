use std::sync::Arc;

use rspack_error::Result;

use super::{
  TaskContext,
  build::BuildTask,
  lazy::{collect_forwarded_ids, request_lazy_dependencies},
};
use crate::{
  BoxModule, BuildContext, DependencyRef, ModuleIdentifier,
  module_graph::{ModuleGraph, ModuleGraphModule},
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct AddTask {
  pub build_context: Arc<BuildContext>,
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
      build_context,
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

    let forwarded_ids = collect_forwarded_ids(
      &mut context.artifact.module_to_lazy_make,
      &context.artifact.module_graph,
      module_identifier,
      &dependencies,
    );

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

      return Ok(
        request_lazy_dependencies(
          &mut context.artifact.module_to_lazy_make,
          &mut context.artifact.module_graph,
          module_identifier,
          forwarded_ids,
        )
        .into_iter()
        .map(|task| Box::new(task) as Box<dyn Task<TaskContext>>)
        .collect(),
      );
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
      build_context,
      module,
      forwarded_ids,
      module_build_cache: context.module_build_cache.clone(),
      value_cache_versions: context.value_cache_versions.clone(),
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
