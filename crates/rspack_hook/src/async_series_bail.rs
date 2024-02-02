use std::{fmt, future::Future};

use async_trait::async_trait;
use rspack_error::Result;

use crate::util::sort_push;

#[async_trait]
pub trait AsyncSeriesBail<Input, Output> {
  async fn run(&self, input: &mut Input) -> Result<Option<Output>>;
  fn stage(&self) -> i32 {
    0
  }
}

#[async_trait]
impl<I, O, F, T> AsyncSeriesBail<I, O> for T
where
  I: Send,
  F: Future<Output = Result<Option<O>>> + Send,
  T: Fn(&mut I) -> F + Sync,
{
  async fn run(&self, input: &mut I) -> Result<Option<O>> {
    self(input).await
  }
}

#[async_trait]
impl<I, O, F, T> AsyncSeriesBail<I, O> for (T, i32)
where
  I: Send,
  F: Future<Output = Result<Option<O>>> + Send,
  T: Fn(&mut I) -> F + Sync,
{
  async fn run(&self, input: &mut I) -> Result<Option<O>> {
    self.0(input).await
  }
  fn stage(&self) -> i32 {
    self.1
  }
}

pub struct AsyncSeriesBailHook<I, O>(Vec<Box<dyn AsyncSeriesBail<I, O> + Send + Sync>>);

impl<I, O> fmt::Debug for AsyncSeriesBailHook<I, O> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeriesBailHook(..)")
  }
}

impl<I, O> Default for AsyncSeriesBailHook<I, O> {
  fn default() -> Self {
    Self(Vec::default())
  }
}

impl<I, O> AsyncSeriesBailHook<I, O> {
  pub async fn call(&self, input: &mut I) -> Result<Option<O>> {
    for hook in &self.0 {
      if let Some(res) = hook.run(input).await? {
        return Ok(Some(res));
      }
    }
    Ok(None)
  }

  pub fn tap(&mut self, hook: impl AsyncSeriesBail<I, O> + 'static + Send + Sync) {
    sort_push(&mut self.0, Box::new(hook), |e| e.stage());
  }
}
