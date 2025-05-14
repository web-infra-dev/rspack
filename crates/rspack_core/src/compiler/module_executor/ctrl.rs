use tokio::sync::mpsc::UnboundedReceiver;

use super::{context::ExecutorTaskContext, entry::EntryTask};
use crate::utils::task_loop::{Task, TaskResult, TaskType};

#[derive(Debug)]
pub enum Event {
  ImportModule(EntryTask),
  Stop,
}

#[derive(Debug)]
pub struct CtrlTask {
  pub event_receiver: UnboundedReceiver<Event>,
}

#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for CtrlTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Async
  }

  async fn background_run(mut self: Box<Self>) -> TaskResult<ExecutorTaskContext> {
    while let Some(event) = self.event_receiver.recv().await {
      tracing::debug!("CtrlTask async receive {:?}", event);
      match event {
        Event::ImportModule(entry_task) => return Ok(vec![Box::new(entry_task), self]),
        Event::Stop => {
          return Ok(vec![]);
        }
      }
    }
    // if channel has been closed, finish this task
    Ok(vec![])
  }
}
