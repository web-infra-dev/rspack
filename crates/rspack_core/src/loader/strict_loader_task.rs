use std::sync::{Arc, Weak};

use super::{Content, LoaderContext, RunnerContext};

pub struct StrictLoaderTask {
  pub owner: u32,
  pub start: u32,
  pub end: u32,
  pub context: Option<Weak<LoaderContext<RunnerContext>>>,
  pub content: Option<Content>,
  // Keep the worker module pointer valid if the waiting runner is cancelled.
  _keep_alive: Arc<LoaderContext<RunnerContext>>,
}

impl StrictLoaderTask {
  pub fn new(
    context: Arc<LoaderContext<RunnerContext>>,
    content: Option<Content>,
    owner: u32,
    start: u32,
    end: u32,
  ) -> Self {
    Self {
      owner,
      start,
      end,
      context: Some(Arc::downgrade(&context)),
      content,
      _keep_alive: context,
    }
  }
}
