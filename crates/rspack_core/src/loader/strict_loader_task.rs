use rspack_error::{Result, error};
use tokio::sync::oneshot;

use super::{LoaderContext, RunnerContext};

pub type StrictLoaderResult = (Box<LoaderContext<RunnerContext>>, Vec<String>, Result<()>);

pub struct StrictLoaderTask {
  pub owner: u32,
  pub start: u32,
  pub end: u32,
  state: Option<(
    Box<LoaderContext<RunnerContext>>,
    oneshot::Sender<StrictLoaderResult>,
  )>,
}

impl StrictLoaderTask {
  pub fn new(
    context: Box<LoaderContext<RunnerContext>>,
    owner: u32,
    start: u32,
    end: u32,
    sender: oneshot::Sender<StrictLoaderResult>,
  ) -> Self {
    Self {
      owner,
      start,
      end,
      state: Some((context, sender)),
    }
  }

  pub fn context_mut(&mut self) -> &mut LoaderContext<RunnerContext> {
    &mut self
      .state
      .as_mut()
      .expect("unfinished strict loader task")
      .0
  }

  pub fn finish(mut self, pitches: Vec<String>, result: Result<()>) {
    if let Some((context, sender)) = self.state.take() {
      let _ = sender.send((context, pitches, result));
    }
  }
}

impl Drop for StrictLoaderTask {
  fn drop(&mut self) {
    if let Some((context, sender)) = self.state.take() {
      let _ = sender.send((
        context,
        Vec::new(),
        Err(error!("Strict loader task was cancelled")),
      ));
    }
  }
}
