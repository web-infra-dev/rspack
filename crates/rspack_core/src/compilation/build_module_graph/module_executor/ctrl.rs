use tokio::sync::mpsc::UnboundedReceiver;

use super::{context::ExecutorTaskContext, entry::EntryTask};
use crate::utils::task_loop::{Task, TaskResult, TaskType};

/// Event for CtrlTask
#[derive(Debug)]
pub enum Event {
  /// Trigger a import module task
  ImportModule(EntryTask),
  /// Stop
  Stop,
}

/// A background task to make task loop without exit and dynamically add entry tasks.
#[derive(Debug)]
pub struct CtrlTask {
  pub event_receiver: UnboundedReceiver<Event>,
}

#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for CtrlTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }

  async fn background_run(mut self: Box<Self>) -> TaskResult<ExecutorTaskContext> {
    let Some(event) = self.event_receiver.recv().await else {
      // if channel has been closed, finish this task
      return Ok(vec![]);
    };
    tracing::debug!("CtrlTask async receive {:?}", event);
    match event {
      // return self to keep CtrlTask still run.
      Event::ImportModule(entry_task) => return Ok(vec![Box::new(entry_task), self]),
      Event::Stop => {
        return Ok(vec![]);
      }
    }
  }
}
