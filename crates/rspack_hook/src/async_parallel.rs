use std::{fmt, future::Future};

use async_trait::async_trait;
use futures_concurrency::prelude::*;

use crate::util::sort_push;

#[async_trait]
pub trait AsyncParallel<Input> {
  async fn run(&self, input: &Input);
  fn stage(&self) -> i32 {
    0
  }
}

#[async_trait]
impl<I, F, T> AsyncParallel<I> for T
where
  I: Send + Sync,
  F: Future<Output = ()> + Send,
  T: Fn(&I) -> F + Sync,
{
  async fn run(&self, input: &I) {
    self(input).await;
  }
}

#[async_trait]
impl<I, F, T> AsyncParallel<I> for (T, i32)
where
  I: Send + Sync,
  F: Future<Output = ()> + Send,
  T: Fn(&I) -> F + Sync,
{
  async fn run(&self, input: &I) {
    self.0(input).await;
  }
  fn stage(&self) -> i32 {
    self.1
  }
}

pub struct AsyncParallelHook<I>(Vec<Box<dyn AsyncParallel<I> + Send + Sync>>);

impl<I> fmt::Debug for AsyncParallelHook<I> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncParallelHook(..)")
  }
}

impl<I> Default for AsyncParallelHook<I> {
  fn default() -> Self {
    Self(Vec::default())
  }
}

impl<I> AsyncParallelHook<I> {
  pub async fn call(&self, input: &I) {
    let groupes = self.0.group_by(|a, b| a.stage() == b.stage());
    for group in groupes {
      let futs: Vec<_> = group.iter().map(|hook| hook.run(input)).collect();
      futs.join().await;
    }
  }

  pub fn tap(&mut self, hook: impl AsyncParallel<I> + 'static + Send + Sync) {
    sort_push(&mut self.0, Box::new(hook), |e| e.stage());
  }
}
