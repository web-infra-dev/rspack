use std::fmt;

use async_trait::async_trait;
use futures_concurrency::prelude::*;
use rspack_error::Result;
use rustc_hash::FxHashSet;

use crate::{Hook, Interceptor};

#[async_trait]
pub trait AsyncParallel<I> {
  async fn run(&self, input: &I) -> Result<()>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncParallelHook<I> {
  taps: Vec<Box<dyn AsyncParallel<I> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I> Hook for AsyncParallelHook<I> {
  type Tap = Box<dyn AsyncParallel<I> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I> fmt::Debug for AsyncParallelHook<I> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncParallelHook(..)")
  }
}

impl<I> Default for AsyncParallelHook<I> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I> AsyncParallelHook<I> {
  pub async fn call(&self, input: &I) -> Result<()> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call(self).await?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    let futs: Vec<_> = all_taps.iter().map(|t| t.run(input)).collect();
    futs.try_join().await?;
    Ok(())
  }

  pub fn tap(&mut self, hook: impl AsyncParallel<I> + Send + Sync + 'static) {
    self.taps.push(Box::new(hook));
  }
}

#[async_trait]
pub trait AsyncParallel3<I1, I2, I3> {
  async fn run(&self, input1: &I1, input2: &I2, input3: &I3) -> Result<()>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncParallel3Hook<I1, I2, I3> {
  taps: Vec<Box<dyn AsyncParallel3<I1, I2, I3> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I1, I2, I3> Hook for AsyncParallel3Hook<I1, I2, I3> {
  type Tap = Box<dyn AsyncParallel3<I1, I2, I3> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I1, I2, I3> fmt::Debug for AsyncParallel3Hook<I1, I2, I3> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncParallel3Hook(..)")
  }
}

impl<I1, I2, I3> Default for AsyncParallel3Hook<I1, I2, I3> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1, I2, I3> AsyncParallel3Hook<I1, I2, I3> {
  pub async fn call(&self, input1: &I1, input2: &I2, input3: &I3) -> Result<()> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call(self).await?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    let futs: Vec<_> = all_taps
      .iter()
      .map(|t| t.run(input1, input2, input3))
      .collect();
    futs.try_join().await?;
    Ok(())
  }

  pub fn tap(&mut self, hook: impl AsyncParallel3<I1, I2, I3> + Send + Sync + 'static) {
    self.taps.push(Box::new(hook));
  }
}
