use std::collections::hash_map::Entry;

use super::{
  super::graph_updater::repair::{context::TaskContext, factorize::FactorizeTask},
  context::{ExecutorTaskContext, ImportModuleMeta},
  execute::ExecuteTask,
  overwrite::overwrite_tasks,
};
use crate::{
  Context, Dependency, LoaderImportDependency,
  utils::task_loop::{Task, TaskResult, TaskType},
};

/// A task for generate import module entry.
#[derive(Debug)]
pub struct EntryTask {
  pub meta: ImportModuleMeta,
  pub origin_module_context: Option<Context>,
  pub execute_task: ExecuteTask,
}
#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for EntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(
    self: Box<Self>,
    context: &mut ExecutorTaskContext,
  ) -> TaskResult<ExecutorTaskContext> {
    let Self {
      meta,
      origin_module_context,
      execute_task,
    } = *self;
    let ExecutorTaskContext {
      entries,
      origin_context,
      tracker,
      executed_entry_deps,
    } = context;

    let mut res = vec![];
    let (dep_id, is_new) = match entries.entry(meta.clone()) {
      Entry::Vacant(v) => {
        // not exist, generate a new dependency
        let dep = Box::new(LoaderImportDependency::new(
          meta.request.clone(),
          origin_module_context.unwrap_or(Context::from("")),
        ));
        let dep_id = *dep.id();

        let mg = TaskContext::get_module_graph_mut(&mut origin_context.artifact);
        mg.add_dependency(dep.clone());

        res.extend(overwrite_tasks(vec![Box::new(FactorizeTask {
          compiler_id: origin_context.compiler_id,
          compilation_id: origin_context.compilation_id,
          module_factory: origin_context
            .dependency_factories
            .get(dep.dependency_type())
            .unwrap_or_else(|| {
              panic!(
                "should have dependency_factories for dependency_type: {}",
                dep.dependency_type()
              )
            })
            .clone(),
          original_module_identifier: None,
          original_module_source: None,
          issuer: None,
          issuer_layer: meta.layer.clone(),
          original_module_context: None,
          dependencies: vec![dep],
          resolve_options: None,
          options: origin_context.compiler_options.clone(),
          resolver_factory: origin_context.resolver_factory.clone(),
          from_unlazy: false,
        })]));
        (*v.insert(dep_id), true)
      }
      Entry::Occupied(v) => (*v.get(), false),
    };

    // mark as executed
    executed_entry_deps.insert(dep_id);

    if tracker.is_running(&dep_id) {
      let mg = origin_context.artifact.get_module_graph();
      // the module in module executor need to check.
      if mg
        .module_graph_module_by_identifier(&meta.origin_module_identifier)
        .is_some()
      {
        execute_task.finish_with_error(rspack_error::error!(
          "The added task is running, maybe have a circular build dependency. MetaInfo: {:?}",
          meta
        ));
        return Ok(vec![]);
      }
    }

    res.extend(tracker.on_entry(is_new, dep_id, Box::new(execute_task)));

    Ok(res)
  }
}
