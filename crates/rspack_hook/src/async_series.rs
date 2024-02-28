use std::fmt;

use async_trait::async_trait;
use parking_lot::Mutex;
use rspack_error::Result;
use rustc_hash::FxHashSet;

use crate::{interceptor::Interceptor, util::sort_push};

#[async_trait]
pub trait AsyncSeries<Input> {
  async fn run(&self, input: &mut Input) -> Result<()>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncSeriesHook<I>(Vec<Box<dyn AsyncSeries<I> + Send + Sync>>);

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
  pub async fn call(&self, input: &mut I) -> Result<()> {
    for hook in &self.0 {
      hook.run(input).await?;
    }
    Ok(())
  }

  pub fn tap(&mut self, hook: impl AsyncSeries<I> + 'static + Send + Sync) {
    sort_push(&mut self.0, Box::new(hook), |e| e.stage());
  }
}

#[async_trait]
pub trait AsyncSeries2<I1, I2> {
  async fn run(&self, input1: &mut I1, input2: &mut I2) -> Result<()>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncSeries2Hook<I1, I2> {
  taps: Mutex<Vec<Box<dyn AsyncSeries2<I1, I2> + Send + Sync>>>,
  interceptors: Vec<Box<dyn Interceptor<AsyncSeries2Hook<I1, I2>> + Send + Sync>>,
}

impl<I1, I2> fmt::Debug for AsyncSeries2Hook<I1, I2> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeries2Hook(..)")
  }
}

impl<I1, I2> Default for AsyncSeries2Hook<I1, I2> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1, I2> AsyncSeries2Hook<I1, I2> {
  pub async fn call(&self, input1: &mut I1, input2: &mut I2) -> Result<()> {
    for interceptor in self.interceptors.iter() {
      interceptor.call(self).await?;
    }
    let mut taps = self.taps.lock();
    taps.sort_by_key(|hook| hook.stage());
    for tap in taps.iter() {
      tap.run(input1, input2).await?;
    }
    Ok(())
  }

  pub fn tap(&self, tap: Box<dyn AsyncSeries2<I1, I2> + Send + Sync>) {
    self.taps.lock().push(tap);
  }

  pub fn intercept(
    &mut self,
    interceptor: Box<dyn Interceptor<AsyncSeries2Hook<I1, I2>> + Send + Sync>,
  ) {
    self.interceptors.push(interceptor);
  }

  pub fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.lock().iter().map(|h| h.stage()))
  }
}
