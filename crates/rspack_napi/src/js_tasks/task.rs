use std::{any::Any, sync::Arc};

use rspack_error::{Result, error};
use tokio::sync::oneshot;

use super::{LoaderTaskTiming, TimingPoint};

/// The original allocation travels through the queue and back without copying.
/// Type erasure keeps the queue independent of the loader runner's context type.
pub type LoaderTaskResult = (Box<dyn Any + Send>, Vec<String>, Result<()>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoaderTaskMode {
  Main,
  Parallel,
  Pure,
}

pub struct LoaderTask {
  pub timing: Option<Arc<LoaderTaskTiming>>,
  context: Option<Box<dyn Any + Send>>,
  pub main_object_handle: u32,
  pub mode: LoaderTaskMode,
  pub start: u32,
  pub end: u32,
  result: Option<oneshot::Sender<LoaderTaskResult>>,
}

impl LoaderTask {
  pub fn new<T: Send + 'static>(
    context: Box<T>,
    main_object_handle: u32,
    mode: LoaderTaskMode,
    start: u32,
    end: u32,
    result: oneshot::Sender<LoaderTaskResult>,
  ) -> Self {
    Self {
      timing: None,
      context: Some(context),
      main_object_handle,
      mode,
      start,
      end,
      result: Some(result),
    }
  }

  pub fn context<T: 'static>(&self) -> &T {
    self
      .context
      .as_ref()
      .and_then(|value| value.downcast_ref())
      .expect("unfinished loader task owns a context of the consumer's type")
  }

  pub fn context_mut<T: 'static>(&mut self) -> &mut T {
    self
      .context
      .as_mut()
      .and_then(|value| value.downcast_mut())
      .expect("unfinished loader task owns a context of the consumer's type")
  }

  pub fn finish(mut self, pitches: Vec<String>, result: Result<()>) {
    if let (Some(context), Some(sender)) = (self.context.take(), self.result.take()) {
      let _ = sender.send((context, pitches, result));
    }
  }
}

impl Drop for LoaderTask {
  fn drop(&mut self) {
    if let (Some(context), Some(sender)) = (self.context.take(), self.result.take()) {
      let _ = sender.send((
        context,
        Vec::new(),
        Err(error!("Loader task was cancelled")),
      ));
    }
  }
}

/// Each task kind owns its completion protocol. Further JS operations can add a
/// variant without changing channel or subscription ownership.
pub enum JsTask {
  Loader(LoaderTask),
}

impl JsTask {
  pub fn mark_timing(&self, point: TimingPoint) {
    match self {
      Self::Loader(task) => {
        if let Some(timing) = &task.timing {
          timing.mark(point);
        }
      }
    }
  }

  pub fn fail(self, message: &str) {
    match self {
      Self::Loader(task) => task.finish(Vec::new(), Err(error!("{message}"))),
    }
  }
}
