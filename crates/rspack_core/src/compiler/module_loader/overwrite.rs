use super::context::LoadTaskContext;
use crate::{
  compiler::make::repair::{
    add::AddTask, build::BuildResultTask, factorize::FactorizeResultTask, MakeTaskContext,
  },
  utils::task_loop::{Task, TaskResult, TaskType},
};

/// Transform tasks with MakeTaskContext to tasks with ExecutorTaskContext.
pub fn overwrite_tasks(
  tasks: Vec<Box<dyn Task<MakeTaskContext>>>,
) -> Vec<Box<dyn Task<LoadTaskContext>>> {
  tasks
    .into_iter()
    .map(|task| Box::new(OverwriteTask(task)) as Box<dyn Task<LoadTaskContext>>)
    .collect()
}

/// A wrapped task to run MakeTaskContext task.
///
/// This task will intercept the result of the inner task and trigger tracker.on_*
#[derive(Debug)]
pub struct OverwriteTask(Box<dyn Task<MakeTaskContext>>);

#[async_trait::async_trait]
impl Task<LoadTaskContext> for OverwriteTask {
  fn get_task_type(&self) -> TaskType {
    self.0.get_task_type()
  }

  async fn main_run(self: Box<Self>, context: &mut LoadTaskContext) -> TaskResult<LoadTaskContext> {
    let origin_task = self.0;
    let LoadTaskContext {
      origin_context,
      tracker,
      ..
    } = context;
    // factorize result task
    if let Some(factorize_result_task) = origin_task.as_any().downcast_ref::<FactorizeResultTask>()
    {
      let dep_id = *factorize_result_task.dependencies[0].id();
      let mut res = overwrite_tasks(origin_task.main_run(origin_context).await?);
      if res.is_empty() {
        res.extend(tracker.on_factorize_failed(&dep_id));
      }
      return Ok(res);
    }

    // add task
    if let Some(add_task) = origin_task.as_any().downcast_ref::<AddTask>() {
      let dep_id = *add_task.dependencies[0].id();
      let mid = add_task.module.identifier();

      let mut res = overwrite_tasks(origin_task.main_run(origin_context).await?);
      if res.is_empty() {
        let mg = origin_context.artifact.get_module_graph();
        // module exist means it has already been built.
        if mg.module_by_identifier(&mid).is_some() {
          res.extend(tracker.on_add_built_module(&dep_id));
        }
      }
      return Ok(res);
    }

    // build result task
    if let Some(build_result_task) = origin_task.as_any().downcast_ref::<BuildResultTask>() {
      let module_identifier = build_result_task.module.identifier();
      // ignore create sub module tasks
      let _ = origin_task.main_run(origin_context).await?;
      return Ok(tracker.on_build_result(origin_context, &module_identifier));
    }

    // other task
    Ok(overwrite_tasks(origin_task.main_run(origin_context).await?))
  }

  async fn background_run(self: Box<Self>) -> TaskResult<LoadTaskContext> {
    let origin_task = self.0;
    Ok(overwrite_tasks(origin_task.background_run().await?))
  }
}
