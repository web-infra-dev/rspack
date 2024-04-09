use std::fmt;

use rspack_error::Result;
use rustc_hash::FxHashSet;

use crate::{Hook, Interceptor};

pub trait SyncSeries<I1> {
  fn run(&self, input1: &mut I1) -> Result<()>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct SyncSeriesHook<I1> {
  taps: Vec<Box<dyn SyncSeries<I1> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I1> Hook for SyncSeriesHook<I1> {
  type Tap = Box<dyn SyncSeries<I1> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I1> fmt::Debug for SyncSeriesHook<I1> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "SyncSeriesHook(..)")
  }
}

impl<I1> Default for SyncSeriesHook<I1> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1> SyncSeriesHook<I1> {
  pub fn call(&self, input1: &mut I1) -> Result<()> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call_blocking(self)?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    for tap in all_taps {
      tap.run(input1)?;
    }
    Ok(())
  }

  pub fn tap(&mut self, tap: Box<dyn SyncSeries<I1> + Send + Sync>) {
    self.taps.push(tap);
  }
}

pub trait SyncSeries4<I1, I2, I3, I4> {
  fn run(&self, input1: &mut I1, input2: &mut I2, input3: &mut I3, input4: &mut I4) -> Result<()>;
  fn stage(&self) -> i32 {
    0
  }
}

pub struct SyncSeries4Hook<I1, I2, I3, I4> {
  taps: Vec<Box<dyn SyncSeries4<I1, I2, I3, I4> + Send + Sync>>,
  interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl<I1, I2, I3, I4> Hook for SyncSeries4Hook<I1, I2, I3, I4> {
  type Tap = Box<dyn SyncSeries4<I1, I2, I3, I4> + Send + Sync>;

  fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
  }

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static) {
    self.interceptors.push(Box::new(interceptor));
  }
}

impl<I1, I2, I3, I4> fmt::Debug for SyncSeries4Hook<I1, I2, I3, I4> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "SyncSeries4Hook(..)")
  }
}

impl<I1, I2, I3, I4> Default for SyncSeries4Hook<I1, I2, I3, I4> {
  fn default() -> Self {
    Self {
      taps: Default::default(),
      interceptors: Default::default(),
    }
  }
}

impl<I1, I2, I3, I4> SyncSeries4Hook<I1, I2, I3, I4> {
  pub fn call(
    &self,
    input1: &mut I1,
    input2: &mut I2,
    input3: &mut I3,
    input4: &mut I4,
  ) -> Result<()> {
    let mut additional_taps = Vec::new();
    for interceptor in self.interceptors.iter() {
      additional_taps.extend(interceptor.call_blocking(self)?);
    }
    let mut all_taps = Vec::new();
    all_taps.extend(&additional_taps);
    all_taps.extend(&self.taps);
    all_taps.sort_by_key(|hook| hook.stage());
    for tap in all_taps {
      tap.run(input1, input2, input3, input4)?;
    }
    Ok(())
  }

  pub fn tap(&mut self, tap: Box<dyn SyncSeries4<I1, I2, I3, I4> + Send + Sync>) {
    self.taps.push(tap);
  }
}
