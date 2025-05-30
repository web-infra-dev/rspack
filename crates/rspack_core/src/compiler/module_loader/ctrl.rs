use tokio::sync::mpsc::UnboundedReceiver;

use super::{clean::CleanEntryTask, context::LoadTaskContext, entry::EntryTask};
use crate::utils::task_loop::{Task, TaskResult, TaskType};

/// Event for CtrlTask
#[derive(Debug)]
pub enum Event {
  /// Trigger a load module task
  LoadModule(EntryTask),
  /// Stop
  Stop(CleanEntryTask),
}

/// A background task to make task loop without exit and dynamically add entry tasks.
#[derive(Debug)]
pub struct CtrlTask {
  pub event_receiver: UnboundedReceiver<Event>,
}

#[async_trait::async_trait]
impl Task<LoadTaskContext> for CtrlTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }

  async fn background_run(mut self: Box<Self>) -> TaskResult<LoadTaskContext> {
    let Some(event) = self.event_receiver.recv().await else {
      // if channel has been closed, finish this task
      return Ok(vec![]);
    };
    tracing::debug!("CtrlTask async receive {:?}", event);
    match event {
      // return self to keep CtrlTask still run.
      Event::LoadModule(entry_task) => return Ok(vec![Box::new(entry_task), self]),
      Event::Stop(clean_entry_task) => {
        return Ok(vec![Box::new(clean_entry_task)]);
      }
    }
  }
}
