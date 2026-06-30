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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookTapIndex(i32);

impl HookTapIndex {
  pub fn base(index: usize) -> Self {
    debug_assert!(index < i32::MAX as usize);
    Self(index as i32 + 1)
  }

  pub fn additional(index: usize) -> Self {
    debug_assert!(index < i32::MAX as usize);
    Self(-((index as i32) + 1))
  }

  pub fn is_base(self) -> bool {
    self.0 > 0
  }

  pub fn index(self) -> usize {
    if self.is_base() {
      (self.0 - 1) as usize
    } else {
      (-self.0 - 1) as usize
    }
  }
}

pub struct MergedStageIndices<'a> {
  base_stages: &'a [i32],
  additional_stages: &'a [i32],
  additional_order: Vec<usize>,
  base_index: usize,
  additional_cursor: usize,
}

pub fn merged_stage_indices<'a>(
  base_stages: &'a [i32],
  additional_stages: &'a [i32],
) -> MergedStageIndices<'a> {
  MergedStageIndices {
    base_stages,
    additional_stages,
    additional_order: sort_indices_by_stage(additional_stages),
    base_index: 0,
    additional_cursor: 0,
  }
}

impl Iterator for MergedStageIndices<'_> {
  type Item = HookTapIndex;

  fn next(&mut self) -> Option<Self::Item> {
    if self.base_index == self.base_stages.len()
      && self.additional_cursor == self.additional_order.len()
    {
      return None;
    }

    if self.additional_cursor == self.additional_order.len() {
      let index = self.base_index;
      self.base_index += 1;
      return Some(HookTapIndex::base(index));
    }

    if self.base_index == self.base_stages.len() {
      let index = self.additional_order[self.additional_cursor];
      self.additional_cursor += 1;
      return Some(HookTapIndex::additional(index));
    }

    let additional_index = self.additional_order[self.additional_cursor];
    if self.base_stages[self.base_index] <= self.additional_stages[additional_index] {
      let index = self.base_index;
      self.base_index += 1;
      Some(HookTapIndex::base(index))
    } else {
      self.additional_cursor += 1;
      Some(HookTapIndex::additional(additional_index))
    }
  }
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
