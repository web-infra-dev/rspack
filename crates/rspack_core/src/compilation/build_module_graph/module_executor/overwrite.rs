use super::{
  super::graph_updater::repair::{
    add::AddTask, context::TaskContext, factorize::FactorizeResultTask,
    process_dependencies::ProcessDependenciesTask,
  },
  context::ExecutorTaskContext,
};
use crate::utils::task_loop::{Task, TaskResult, TaskType};

/// Transform tasks with TaskContext to tasks with ExecutorTaskContext.
pub fn overwrite_tasks(
  tasks: Vec<Box<dyn Task<TaskContext>>>,
) -> Vec<Box<dyn Task<ExecutorTaskContext>>> {
  tasks
    .into_iter()
    .map(|task| Box::new(OverwriteTask(task)) as Box<dyn Task<ExecutorTaskContext>>)
    .collect()
}

/// A wrapped task to run TaskContext task.
///
/// This task will intercept the result of the inner task and trigger tracker.on_*
#[derive(Debug)]
pub struct OverwriteTask(Box<dyn Task<TaskContext>>);

#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for OverwriteTask {
  fn get_task_type(&self) -> TaskType {
    self.0.get_task_type()
  }

  async fn main_run(
    self: Box<Self>,
    context: &mut ExecutorTaskContext,
  ) -> TaskResult<ExecutorTaskContext> {
    let origin_task = self.0;
    let ExecutorTaskContext {
      origin_context,
      tracker,
      ..
    } = context;
    // factorize result task
    if let Some(factorize_result_task) = origin_task.as_any().downcast_ref::<FactorizeResultTask>()
    {
      let dep_id = *factorize_result_task.dependencies[0].id();
      let original_module_identifier = factorize_result_task.original_module_identifier;
      let mut res = overwrite_tasks(origin_task.main_run(origin_context).await?);
      if res.is_empty() {
        res.extend(tracker.on_factorize_failed(origin_context, original_module_identifier, dep_id))
      }
      return Ok(res);
    }

    // add task
    if let Some(add_task) = origin_task.as_any().downcast_ref::<AddTask>() {
      let dep_id = *add_task.dependencies[0].id();
      let original_module_identifier = add_task.original_module_identifier;
      let target_module_identifier = add_task.module.identifier();

      let mut res = overwrite_tasks(origin_task.main_run(origin_context).await?);
      if res.is_empty() {
        res.extend(tracker.on_add_resolved_module(
          origin_context,
          original_module_identifier,
          dep_id,
          target_module_identifier,
        ));
      } else {
        tracker.on_add(target_module_identifier);
      }
      return Ok(res);
    }

    // process dependencies
    if let Some(process_dependencies_task) = origin_task
      .as_any()
      .downcast_ref::<ProcessDependenciesTask>()
    {
      let original_module_identifier = process_dependencies_task.original_module_identifier;
      let mut res = overwrite_tasks(origin_task.main_run(origin_context).await?);
      res.extend(tracker.on_process_dependencies(
        origin_context,
        original_module_identifier,
        res.len(),
      ));
      return Ok(res);
    }

    // other task
    Ok(overwrite_tasks(origin_task.main_run(origin_context).await?))
  }

  async fn background_run(self: Box<Self>) -> TaskResult<ExecutorTaskContext> {
    let origin_task = self.0;
    Ok(overwrite_tasks(origin_task.background_run().await?))
  }
}
