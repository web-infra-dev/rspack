use std::fmt;

use async_trait::async_trait;
use futures_concurrency::prelude::*;
use rspack_error::Result;

use crate::util::sort_push;

#[async_trait]
pub trait AsyncParallel<Input> {
  async fn run(&self, input: &Input) -> Result<()>;
  fn stage(&self) -> i32 {
    0
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
  pub async fn call(&self, input: &I) -> Result<()> {
    let groups = self.0.slice().group_by(|a, b| a.stage() == b.stage());
    for group in groups {
      let futs: Vec<_> = group.iter().map(|hook| hook.run(input)).collect();
      futs.try_join().await?;
    }
    Ok(())
  }

  pub fn tap(&mut self, hook: Box<dyn AsyncParallel<I> + Send + Sync>) {
    sort_push(&mut self.0, hook, |e| e.stage());
  }
}
