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
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
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

  pub fn tap(&mut self, tap: Box<dyn AsyncSeriesBail<I, O> + Send + Sync>) {
    self.taps.push(tap);
  }
}

#[async_trait]
pub trait AsyncSeriesBail2<I1, I2, Output> {
  async fn run(&self, input1: &mut I1, input2: &mut I2) -> Result<Option<Output>>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncSeriesBail2Hook<I1, I2, O> {
  taps: Vec<Box<dyn AsyncSeriesBail2<I1, I2, O> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I1, I2, O> Hook for AsyncSeriesBail2Hook<I1, I2, O> {
  type Tap = Box<dyn AsyncSeriesBail2<I1, I2, O> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I1, I2, O> fmt::Debug for AsyncSeriesBail2Hook<I1, I2, O> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeriesBail2Hook(..)")
  }
}

impl<I1, I2, O> Default for AsyncSeriesBail2Hook<I1, I2, O> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1, I2, O> AsyncSeriesBail2Hook<I1, I2, O> {
  pub async fn call(&self, input1: &mut I1, input2: &mut I2) -> Result<Option<O>> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call(self).await?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    for hook in all_taps {
      if let Some(res) = hook.run(input1, input2).await? {
        return Ok(Some(res));
      }
    }
    Ok(None)
  }

  pub fn tap(&mut self, tap: Box<dyn AsyncSeriesBail2<I1, I2, O> + Send + Sync>) {
    self.taps.push(tap);
  }
}

#[async_trait]
pub trait AsyncSeriesBail3<I1, I2, I3, Output> {
  async fn run(&self, input1: &mut I1, input2: &mut I2, input3: &mut I3) -> Result<Option<Output>>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncSeriesBail3Hook<I1, I2, I3, O> {
  taps: Vec<Box<dyn AsyncSeriesBail3<I1, I2, I3, O> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I1, I2, I3, O> Hook for AsyncSeriesBail3Hook<I1, I2, I3, O> {
  type Tap = Box<dyn AsyncSeriesBail3<I1, I2, I3, O> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I1, I2, I3, O> fmt::Debug for AsyncSeriesBail3Hook<I1, I2, I3, O> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeriesBail3Hook(..)")
  }
}

impl<I1, I2, I3, O> Default for AsyncSeriesBail3Hook<I1, I2, I3, O> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1, I2, I3, O> AsyncSeriesBail3Hook<I1, I2, I3, O> {
  pub async fn call(&self, input1: &mut I1, input2: &mut I2, input3: &mut I3) -> Result<Option<O>> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call(self).await?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    for hook in all_taps {
      if let Some(res) = hook.run(input1, input2, input3).await? {
        return Ok(Some(res));
      }
    }
    Ok(None)
  }

  pub fn tap(&mut self, tap: Box<dyn AsyncSeriesBail3<I1, I2, I3, O> + Send + Sync>) {
    self.taps.push(tap);
  }
}

#[async_trait]
pub trait AsyncSeriesBail4<I1, I2, I3, I4, Output> {
  async fn run(
    &self,
    input1: &mut I1,
    input2: &mut I2,
    input3: &mut I3,
    input4: &mut I4,
  ) -> Result<Option<Output>>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct AsyncSeriesBail4Hook<I1, I2, I3, I4, O> {
  taps: Vec<Box<dyn AsyncSeriesBail4<I1, I2, I3, I4, O> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I1, I2, I3, I4, O> Hook for AsyncSeriesBail4Hook<I1, I2, I3, I4, O> {
  type Tap = Box<dyn AsyncSeriesBail4<I1, I2, I3, I4, O> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I1, I2, I3, I4, O> fmt::Debug for AsyncSeriesBail4Hook<I1, I2, I3, I4, O> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AsyncSeriesBail4Hook(..)")
  }
}

impl<I1, I2, I3, I4, O> Default for AsyncSeriesBail4Hook<I1, I2, I3, I4, O> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1, I2, I3, I4, O> AsyncSeriesBail4Hook<I1, I2, I3, I4, O> {
  pub async fn call(
    &self,
    input1: &mut I1,
    input2: &mut I2,
    input3: &mut I3,
    input4: &mut I4,
  ) -> Result<Option<O>> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call(self).await?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    for hook in all_taps {
      if let Some(res) = hook.run(input1, input2, input3, input4).await? {
        return Ok(Some(res));
      }
    }
    Ok(None)
  }

  pub fn tap(&mut self, tap: Box<dyn AsyncSeriesBail4<I1, I2, I3, I4, O> + Send + Sync>) {
    self.taps.push(tap);
  }
}
