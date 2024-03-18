use std::fmt;

use async_trait::async_trait;
use rspack_error::Result;
use rustc_hash::FxHashSet;

use crate::{Hook, Interceptor};

#[async_trait]
pub trait AsyncSeriesBail<Input, Output> {
  async fn run(&self, input: &mut Input) -> Result<Option<Output>>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncSeriesBailHook<I, O> {
  taps: Vec<Box<dyn AsyncSeriesBail<I, O> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<AsyncSeriesBailHook<I, O>> + Send + Sync>>,
}

impl<I, O> Hook for AsyncSeriesBailHook<I, O> {
  type Tap = Box<dyn AsyncSeriesBail<I, O> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I, O> fmt::Debug for AsyncSeriesBailHook<I, O> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeriesBailHook(..)")
  }
}

impl<I, O> Default for AsyncSeriesBailHook<I, O> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I, O> AsyncSeriesBailHook<I, O> {
  pub async fn call(&self, input: &mut I) -> Result<Option<O>> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call(self).await?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    for hook in all_taps {
      if let Some(res) = hook.run(input).await? {
        return Ok(Some(res));
      }
    }
    Ok(None)
  }
}
