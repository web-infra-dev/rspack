use super::context::ExecutorTaskContext;
use crate::{
  compiler::make::repair::{
    add::AddTask, factorize::FactorizeResultTask, process_dependencies::ProcessDependenciesTask,
    MakeTaskContext,
  },
  utils::task_loop::{Task, TaskResult, TaskType},
};

pub fn overwrite_tasks(
  tasks: Vec<Box<dyn Task<MakeTaskContext>>>,
) -> Vec<Box<dyn Task<ExecutorTaskContext>>> {
  tasks
    .into_iter()
    .map(|task| Box::new(OverwriteTask(task)) as Box<dyn Task<ExecutorTaskContext>>)
    .collect()
}

#[derive(Debug)]
pub struct OverwriteTask(Box<dyn Task<MakeTaskContext>>);

impl OverwriteTask {
  fn into_origin_task(self) -> Box<dyn Task<MakeTaskContext>> {
    self.0
  }
}

#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for OverwriteTask {
  fn get_task_type(&self) -> TaskType {
    self.0.get_task_type()
  }

  async fn main_run(
    self: Box<Self>,
    context: &mut ExecutorTaskContext,
  ) -> TaskResult<ExecutorTaskContext> {
    let origin_task = self.into_origin_task();
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
    let origin_task = self.into_origin_task();
    Ok(overwrite_tasks(origin_task.background_run().await?))
  }
}
