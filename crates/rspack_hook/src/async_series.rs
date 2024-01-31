use std::{fmt, future::Future};

use async_trait::async_trait;

use crate::util::sort_push;

#[async_trait]
pub trait AsyncSeries<Input> {
  async fn run(&self, input: &mut Input);
  fn stage(&self) -> i32 {
    0
  }
}

#[async_trait]
impl<I, F, T> AsyncSeries<I> for T
where
  I: Send,
  F: Future<Output = ()> + Send,
  T: Fn(&mut I) -> F + Sync,
{
  async fn run(&self, input: &mut I) {
    self(input).await;
  }
}

#[async_trait]
impl<I, F, T> AsyncSeries<I> for (T, i32)
where
  I: Send,
  F: Future<Output = ()> + Send,
  T: Fn(&mut I) -> F + Sync,
{
  async fn run(&self, input: &mut I) {
    self.0(input).await;
  }
  fn stage(&self) -> i32 {
    self.1
  }
}

pub struct AsyncSeriesHook<I>(Vec<Box<dyn AsyncSeries<I>>>);

impl<I> fmt::Debug for AsyncSeriesHook<I> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeriesHook(..)")
  }
}

impl<I> Default for AsyncSeriesHook<I> {
  fn default() -> Self {
    Self(Vec::default())
  }
}

impl<I> AsyncSeriesHook<I> {
  pub async fn call(&self, input: &mut I) {
    for hook in &self.0 {
      hook.run(input).await;
    }
  }

  pub fn tap(&mut self, hook: impl AsyncSeries<I> + 'static) {
    sort_push(&mut self.0, Box::new(hook), |e| e.stage());
  }
}
