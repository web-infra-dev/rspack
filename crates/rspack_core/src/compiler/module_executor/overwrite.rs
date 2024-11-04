use tokio::sync::mpsc::UnboundedSender;

use super::ctrl::Event;
use crate::{
  compiler::make::repair::{
    add::AddTask, process_dependencies::ProcessDependenciesTask, MakeTaskContext,
  },
  utils::task_loop::{Task, TaskResult, TaskType},
};

/**
Use this task to check module state during make,
it is a proxy task
*/
#[derive(Debug)]
pub struct OverwriteTask {
  pub origin_task: Box<dyn Task<MakeTaskContext>>,
  pub event_sender: UnboundedSender<Event>,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for OverwriteTask {
  fn get_task_type(&self) -> TaskType {
    self.origin_task.get_task_type()
  }

  async fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self {
      origin_task,
      event_sender,
    } = *self;
    // process dependencies
    if let Some(process_dependencies_task) = origin_task
      .as_any()
      .downcast_ref::<ProcessDependenciesTask>()
    {
      let original_module_identifier = process_dependencies_task.original_module_identifier;
      let res = origin_task.sync_run(context).await?;
      if res.is_empty() {
        event_sender
          .send(Event::FinishModule(original_module_identifier))
          .expect("should success");
      } else {
        event_sender
          .send(Event::ProcessDeps(original_module_identifier, res.len()))
          .expect("should success");
      }

      return Ok(res);
    }

    // add task
    if let Some(add_task) = origin_task.as_any().downcast_ref::<AddTask>() {
      let is_self_module = add_task.module.as_self_module().is_some();
      let original_module_identifier = add_task.original_module_identifier;
      let target_module_identifier = add_task.module.identifier();
      let dep_id = *add_task.dependencies[0].id();

      let res = origin_task.sync_run(context).await?;

      if !res.is_empty() || is_self_module {
        event_sender
          .send(Event::Add(
            original_module_identifier,
            target_module_identifier,
            dep_id,
            is_self_module,
          ))
          .expect("should success");
      } else {
        event_sender
          .send(Event::FinishModule(target_module_identifier))
          .expect("should success");
      }

      return Ok(res);
    }

    // other task
    origin_task.sync_run(context).await
  }

  async fn async_run(self: Box<Self>) -> TaskResult<MakeTaskContext> {
    self.origin_task.async_run().await
  }
}
