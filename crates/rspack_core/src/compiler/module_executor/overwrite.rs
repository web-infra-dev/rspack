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

    let is_process_dependencies = origin_task
      .as_any()
      .downcast_ref::<ProcessDependenciesTask>()
      .is_some();
    let is_factorize_result = origin_task
      .as_any()
      .downcast_ref::<FactorizeResultTask>()
      .is_some();
    let is_add_task = origin_task.as_any().downcast_ref::<AddTask>().is_some();

    let res: Vec<_> = origin_task
      .sync_run(context)?
      .into_iter()
      .map(|task| {
        Box::new(OverwriteTask {
          origin_task: task,
          event_sender: event_sender.clone(),
        }) as Box<dyn Task<MakeTaskContext>>
      })
      .collect();

    // process dependencies
    if is_process_dependencies {
      dbg!(res.len());
      event_sender
        .send(Event::FinishModule(res.len()))
        .expect("should success");
    }

    // factorize result task
    if is_factorize_result && res.is_empty() {
      event_sender
        .send(Event::FinishDeps)
        .expect("should success");
    }

    // add task
    if is_add_task && res.is_empty() {
      event_sender
        .send(Event::FinishDeps)
        .expect("should success");
    }

    Ok(res)
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
