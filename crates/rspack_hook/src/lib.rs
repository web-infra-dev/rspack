use async_trait::async_trait;
use rspack_error::Result;
use rustc_hash::FxHashSet;

pub struct HookMetadata {
  pub name: &'static str,
}

pub struct HookCommon {
  metadata: HookMetadata,
  tap_stages: Vec<i32>,
  interceptor_count: usize,
}

impl HookCommon {
  pub fn new(name: &'static str) -> Self {
    Self {
      metadata: HookMetadata { name },
      tap_stages: Vec::new(),
      interceptor_count: 0,
    }
  }

  pub fn name(&self) -> &'static str {
    self.metadata.name
  }

  pub fn tap_stages(&self) -> &[i32] {
    &self.tap_stages
  }

  pub fn push_tap_stage(&mut self, stage: i32) {
    self.tap_stages.push(stage);
  }

  pub fn insert_tap_stage(&mut self, index: usize, stage: i32) {
    self.tap_stages.insert(index, stage);
  }

  pub fn tap_insert_position(&self, stage: i32) -> usize {
    self.tap_stages.partition_point(|&current| current <= stage)
  }

  pub fn increment_interceptor_count(&mut self) {
    self.interceptor_count += 1;
  }

  pub fn interceptor_count(&self) -> usize {
    self.interceptor_count
  }

  pub fn used_stages(&self) -> FxHashSet<i32> {
    FxHashSet::from_iter(self.tap_stages.iter().copied())
  }

  pub fn is_empty(&self) -> bool {
    self.tap_stages.is_empty() && self.interceptor_count == 0
  }
}

pub fn sort_indices_by_stage(stages: &[i32]) -> Vec<usize> {
  let mut indices: Vec<_> = (0..stages.len()).collect();
  indices.sort_by_key(|&index| (stages[index], index));
  indices
}

#[async_trait]
pub trait Interceptor<H: Hook> {
  async fn call(&self, _hook: &H) -> Result<Vec<<H as Hook>::Tap>> {
    unreachable!("Interceptor::call should only used in async hook")
  }

  fn call_blocking(&self, _hook: &H) -> Result<Vec<<H as Hook>::Tap>> {
    unreachable!("Interceptor::call_blocking should only used in sync hook")
  }
}

pub trait Hook {
  type Tap;

  fn used_stages(&self) -> FxHashSet<i32>;

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static)
  where
    Self: Sized;
}

// pub trait Plugin<HookContainer> {
//   fn apply(&self, hook_container: &mut HookContainer);
// }

#[doc(hidden)]
pub mod __macro_helper {
  pub use async_trait::async_trait;
  pub use rspack_error::Result;
  pub use rustc_hash::FxHashSet;
  pub use tracing;
}

pub use rspack_macros::{define_hook, plugin, plugin_hook};
