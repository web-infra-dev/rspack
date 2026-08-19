use rspack_error::Result;

use super::{TaskContext, build::BuildTask, lazy::process_unlazy_dependencies};
use crate::{
  BoxDependency, BoxModule, ModuleIdentifier,
  compilation::build_module_graph::ForwardedIdSet,
  module_graph::{ModuleGraph, ModuleGraphModule},
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: BoxModule,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxDependency>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for AddTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let module_identifier = self.module.identifier();
    let module_graph = &mut context.artifact.module_graph;

    // reuse module for self referenced module
    if self.module.as_self_module().is_some() {
      let issuer = self
        .module_graph_module
        .issuer()
        .identifier()
        .expect("self module should have issuer");

      set_resolved_module(
        module_graph,
        self.original_module_identifier,
        self.dependencies,
        *issuer,
      )?;

      return Ok(vec![]);
    }

    let forwarded_ids = ForwardedIdSet::from_dependencies(&self.dependencies);

    // reuse module if module is already added by other dependency
    if module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .is_some()
    {
      set_resolved_module(
        module_graph,
        self.original_module_identifier,
        self.dependencies,
        module_identifier,
      )?;

      if self.from_unlazy {
        context
          .artifact
          .affected_modules
          .mark_as_add(&module_identifier);
      }

      if module_graph
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
            module_graph,
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

    module_graph.add_module_graph_module(*self.module_graph_module);

    context
      .exports_info_artifact
      .new_exports_info(module_identifier);

    set_resolved_module(
      module_graph,
      self.original_module_identifier,
      self.dependencies,
      module_identifier,
    )?;

    tracing::trace!("Module added: {}", self.module.identifier());
    context
      .artifact
      .affected_modules
      .mark_as_add(&module_identifier);
    Ok(vec![Box::new(BuildTask {
      compiler_id: context.compiler_id,
      compilation_id: context.compilation_id,
      module: self.module,
      resolver_factory: context.resolver_factory.clone(),
      compiler_options: context.compiler_options.clone(),
      plugin_driver: context.plugin_driver.clone(),
      runtime_template: context.runtime_template.create_module_code_template(),
      fs: context.fs.clone(),
      forwarded_ids,
    })])
  }
}

fn set_resolved_module(
  module_graph: &mut ModuleGraph,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<BoxDependency>,
  module_identifier: ModuleIdentifier,
) -> Result<()> {
  for dependency in dependencies {
    module_graph.set_resolved_module(
      original_module_identifier,
      *dependency.id(),
      module_identifier,
    )?;
    module_graph.add_dependency(dependency);
  }
  Ok(())
}
