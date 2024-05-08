use tokio::sync::mpsc::UnboundedSender;

use super::ctrl::Event;
use crate::{
  compiler::make::repair::{
    add::AddTask, factorize::FactorizeResultTask, process_dependencies::ProcessDependenciesTask,
    MakeTaskContext,
  },
  utils::task_loop::{Task, TaskResult, TaskType},
};

pub struct OverwriteTask {
  pub origin_task: Box<dyn Task<MakeTaskContext>>,
  pub event_sender: UnboundedSender<Event>,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for OverwriteTask {
  fn get_task_type(&self) -> TaskType {
    self.origin_task.get_task_type()
  }

  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
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
      let res = origin_task.sync_run(context)?;
      event_sender
        .send(Event::FinishModule(original_module_identifier, res.len()))
        .expect("should success");
      return Ok(res);
    }

    // factorize result task
    if let Some(factorize_result_task) = origin_task.as_any().downcast_ref::<FactorizeResultTask>()
    {
      let dep_id = factorize_result_task
        .dependencies
        .first()
        .cloned()
        .expect("should have dep_id");
      let original_module_identifier = factorize_result_task.original_module_identifier;
      let res = origin_task.sync_run(context)?;
      if res.is_empty() {
        event_sender
          .send(Event::FinishDeps(original_module_identifier, dep_id, None))
          .expect("should success");
      }
      return Ok(res);
    }
    // add task
    if let Some(add_task) = origin_task.as_any().downcast_ref::<AddTask>() {
      let dep_id = add_task
        .dependencies
        .first()
        .cloned()
        .expect("should have dep_id");
      let original_module_identifier = add_task.original_module_identifier;
      let target_module_identifier = add_task.module.identifier();

      let res = origin_task.sync_run(context)?;
      if res.is_empty() {
        event_sender
          .send(Event::FinishDeps(
            original_module_identifier,
            dep_id,
            Some(target_module_identifier),
          ))
          .expect("should success");
      } else {
        event_sender
          .send(Event::StartBuild(target_module_identifier))
          .expect("should success");
      }
      return Ok(res);
    }

    // other task
    origin_task.sync_run(context)
  }

  async fn async_run(self: Box<Self>) -> TaskResult<MakeTaskContext> {
    self.origin_task.async_run().await
  }
}
