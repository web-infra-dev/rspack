use tokio::sync::mpsc::UnboundedSender;

use super::ctrl::Event;
use crate::{
  compiler::make::repair::{
    add::AddTask, factorize::FactorizeResultTask, process_dependencies::ProcessDependenciesTask,
    MakeTaskContext,
  },
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct OverwriteTask {
  pub origin_task: Box<dyn Task<MakeTaskContext>>,
  pub event_sender: UnboundedSender<Event>,
}

fn to_overwrite_task(
  origin_task: Box<dyn Task<MakeTaskContext>>,
  context: &mut MakeTaskContext,
  event_sender: UnboundedSender<Event>,
) -> TaskResult<MakeTaskContext> {
  Ok(
    origin_task
      .sync_run(context)?
      .into_iter()
      .map(|task| {
        Box::new(OverwriteTask {
          origin_task: task,
          event_sender: event_sender.clone(),
        }) as Box<dyn Task<MakeTaskContext>>
      })
      .collect(),
  )
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
      let module_id = process_dependencies_task.original_module_identifier;
      let res = to_overwrite_task(origin_task, context, event_sender.clone())?;
      event_sender
        .send(Event::FinishModule(module_id, res.len()))
        .expect("should success");
      return Ok(res);
    }

    // factorize result task
    if origin_task
      .as_any()
      .downcast_ref::<FactorizeResultTask>()
      .is_some()
    {
      let res = to_overwrite_task(origin_task, context, event_sender.clone())?;
      if res.is_empty() {
        event_sender
          .send(Event::FinishDeps(None))
          .expect("should success");
      }
      return Ok(res);
    }

    // add task
    if let Some(add_task) = origin_task.as_any().downcast_ref::<AddTask>() {
      let module_id = add_task.module.identifier();
      let res = to_overwrite_task(origin_task, context, event_sender.clone())?;
      if res.is_empty() {
        event_sender
          .send(Event::FinishDeps(Some(module_id)))
          .expect("should success");
      } else {
        event_sender
          .send(Event::StartBuild(module_id))
          .expect("should success");
      }
      return Ok(res);
    }

    to_overwrite_task(origin_task, context, event_sender)
  }

  async fn async_run(self: Box<Self>) -> TaskResult<MakeTaskContext> {
    Ok(
      self
        .origin_task
        .async_run()
        .await?
        .into_iter()
        .map(|task| {
          Box::new(OverwriteTask {
            origin_task: task,
            event_sender: self.event_sender.clone(),
          }) as Box<dyn Task<MakeTaskContext>>
        })
        .collect(),
    )
  }
}
