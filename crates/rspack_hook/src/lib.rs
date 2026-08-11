use std::{
  any::{Any, TypeId},
  future::Future,
  pin::Pin,
  sync::Arc,
};

use async_trait::async_trait;
use rspack_error::{Result, error};

type ErasedJsTapRegisterOwner = Arc<dyn Any + Send + Sync>;
#[doc(hidden)]
pub type ErasedJsTap = Box<dyn Any + Send + Sync>;
type AsyncJsTapRegisterFn = Box<
  dyn Fn(
      ErasedJsTapRegisterOwner,
      Vec<i32>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ErasedJsTap>>> + Send>>
    + Send
    + Sync,
>;
type BlockingJsTapRegisterFn =
  Box<dyn Fn(ErasedJsTapRegisterOwner, Vec<i32>) -> Result<Vec<ErasedJsTap>> + Send + Sync>;
type LoadJsTapCountFn = Box<dyn Fn(&dyn Any) -> usize + Send + Sync>;

enum JsTapRegisterFn {
  Async(AsyncJsTapRegisterFn),
  Blocking(BlockingJsTapRegisterFn),
}

pub struct JsTapRegister {
  owner: ErasedJsTapRegisterOwner,
  tap_type_id: TypeId,
  load_tap_count: LoadJsTapCountFn,
  function: JsTapRegisterFn,
}

impl JsTapRegister {
  pub fn new_async<O, T, F, Fut>(
    owner: Arc<O>,
    load_tap_count: fn(&O) -> usize,
    function: F,
  ) -> Self
  where
    O: Any + Send + Sync,
    T: Any + Send + Sync,
    F: Fn(Arc<O>, Vec<i32>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<T>>> + Send + 'static,
  {
    Self {
      owner,
      tap_type_id: TypeId::of::<T>(),
      load_tap_count: Box::new(move |owner| {
        load_tap_count(
          owner
            .downcast_ref::<O>()
            .expect("JsTapRegister owner type should match"),
        )
      }),
      function: JsTapRegisterFn::Async(Box::new(move |owner, stages| {
        let owner = Arc::downcast::<O>(owner).expect("JsTapRegister owner type should match");
        let future = function(owner, stages);
        Box::pin(async move {
          Ok(
            future
              .await?
              .into_iter()
              .map(|tap| Box::new(tap) as ErasedJsTap)
              .collect(),
          )
        })
      })),
    }
  }

  pub fn new_blocking<O, T, F>(owner: Arc<O>, load_tap_count: fn(&O) -> usize, function: F) -> Self
  where
    O: Any + Send + Sync,
    T: Any + Send + Sync,
    F: Fn(Arc<O>, Vec<i32>) -> Result<Vec<T>> + Send + Sync + 'static,
  {
    Self {
      owner,
      tap_type_id: TypeId::of::<T>(),
      load_tap_count: Box::new(move |owner| {
        load_tap_count(
          owner
            .downcast_ref::<O>()
            .expect("JsTapRegister owner type should match"),
        )
      }),
      function: JsTapRegisterFn::Blocking(Box::new(move |owner, stages| {
        Ok(
          function(
            Arc::downcast::<O>(owner).expect("JsTapRegister owner type should match"),
            stages,
          )?
          .into_iter()
          .map(|tap| Box::new(tap) as ErasedJsTap)
          .collect(),
        )
      })),
    }
  }

  fn tap_count(&self) -> usize {
    (self.load_tap_count)(self.owner.as_ref())
  }

  fn is_empty(&self) -> bool {
    self.tap_count() == 0
  }

  fn has_tap_type<T: Any>(&self) -> bool {
    self.tap_type_id == TypeId::of::<T>()
  }

  async fn call(&self, stages: Vec<i32>) -> Result<Vec<ErasedJsTap>> {
    match &self.function {
      JsTapRegisterFn::Async(function) => function(self.owner.clone(), stages).await,
      JsTapRegisterFn::Blocking(_) => Err(error!(
        "blocking JS tap register cannot be called from an async hook"
      )),
    }
  }

  fn call_blocking(&self, stages: Vec<i32>) -> Result<Vec<ErasedJsTap>> {
    match &self.function {
      JsTapRegisterFn::Async(_) => Err(error!(
        "async JS tap register cannot be called from a blocking hook"
      )),
      JsTapRegisterFn::Blocking(function) => function(self.owner.clone(), stages),
    }
  }
}

pub fn unerase_js_taps<T: Any>(taps: Vec<ErasedJsTap>) -> Result<Vec<T>> {
  taps
    .into_iter()
    .map(|tap| {
      tap
        .downcast::<T>()
        .map(|tap| *tap)
        .map_err(|_| error!("JS tap register returned an unexpected tap type"))
    })
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookCallMode {
  Empty,
  RustTaps,
  AdditionalTaps,
}

pub struct HookMetadata {
  pub name: &'static str,
}

pub struct HookCommon {
  metadata: HookMetadata,
  tap_stages: Vec<i32>,
  interceptor_count: usize,
  js_tap_register: Option<JsTapRegister>,
}

impl HookCommon {
  pub fn new(name: &'static str) -> Self {
    Self {
      metadata: HookMetadata { name },
      tap_stages: Vec::new(),
      interceptor_count: 0,
      js_tap_register: None,
    }
  }

  pub fn name(&self) -> &'static str {
    self.metadata.name
  }

  pub fn tap_stages(&self) -> &[i32] {
    &self.tap_stages
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

  pub fn used_stages(&self) -> Vec<i32> {
    let mut used_stages = self.tap_stages.clone();
    // tap_stages is kept sorted by stage, so duplicate stages are adjacent.
    used_stages.dedup();
    used_stages
  }

  pub fn load_js_tap_register<T: Any>(&mut self, register: JsTapRegister) -> Result<()> {
    if self.js_tap_register.is_some() {
      return Err(error!(
        "JS tap register for hook {} has already been loaded",
        self.name()
      ));
    }
    if !register.has_tap_type::<T>() {
      return Err(error!(
        "JS tap register type does not match hook {}",
        self.name()
      ));
    }
    self.js_tap_register = Some(register);
    Ok(())
  }

  pub fn has_js_taps(&self) -> bool {
    self
      .js_tap_register
      .as_ref()
      .is_some_and(|register| !register.is_empty())
  }

  pub fn is_empty(&self) -> bool {
    self.tap_stages.is_empty() && self.interceptor_count == 0 && !self.has_js_taps()
  }

  pub fn call_mode(&self) -> HookCallMode {
    let has_js_taps = self.has_js_taps();
    if self.tap_stages.is_empty() && self.interceptor_count == 0 && !has_js_taps {
      HookCallMode::Empty
    } else if self.interceptor_count == 0 && !has_js_taps {
      HookCallMode::RustTaps
    } else {
      HookCallMode::AdditionalTaps
    }
  }

  pub async fn call_js_tap_register(&self) -> Result<Vec<ErasedJsTap>> {
    if !self.has_js_taps() {
      return Ok(Vec::new());
    }
    self
      .js_tap_register
      .as_ref()
      .expect("JS tap register should exist when JS taps are present")
      .call(self.used_stages())
      .await
  }

  pub fn call_js_tap_register_blocking(&self) -> Result<Vec<ErasedJsTap>> {
    if !self.has_js_taps() {
      return Ok(Vec::new());
    }
    self
      .js_tap_register
      .as_ref()
      .expect("JS tap register should exist when JS taps are present")
      .call_blocking(self.used_stages())
  }
}

pub fn sort_indices_by_stage(stages: &[i32]) -> Vec<u16> {
  debug_assert!(stages.len() <= HookTapIndex::INDEX_LIMIT);
  let mut indices: Vec<_> = (0..stages.len()).map(|index| index as u16).collect();
  indices.sort_by_key(|&index| {
    let index = index as usize;
    (stages[index], index)
  });
  debug_assert!(indices.windows(2).all(|indices| {
    let prev = indices[0] as usize;
    let next = indices[1] as usize;
    (stages[prev], prev) <= (stages[next], next)
  }));
  indices
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookTapIndex(u16);

impl HookTapIndex {
  const INTERCEPT_FLAG: u16 = 1 << 15;
  const INDEX_MASK: u16 = !Self::INTERCEPT_FLAG;
  const INDEX_LIMIT: usize = Self::INDEX_MASK as usize + 1;

  pub fn tap(index: u16) -> Self {
    debug_assert!(index <= Self::INDEX_MASK);
    Self(index)
  }

  pub fn intercept(index: u16) -> Self {
    debug_assert!(index <= Self::INDEX_MASK);
    Self(Self::INTERCEPT_FLAG | index)
  }

  pub fn is_tap(self) -> bool {
    self.0 & Self::INTERCEPT_FLAG == 0
  }

  pub fn index(self) -> usize {
    (self.0 & Self::INDEX_MASK) as usize
  }
}

pub struct MergedTapIndicesByStage<'a> {
  base_stages: &'a [i32],
  additional_stages: &'a [i32],
  additional_order: Vec<u16>,
  base_index: u16,
  additional_cursor: u16,
}

pub fn merged_tap_indices_by_stage<'a>(
  base_stages: &'a [i32],
  additional_stages: &'a [i32],
) -> MergedTapIndicesByStage<'a> {
  debug_assert!(base_stages.len() <= HookTapIndex::INDEX_LIMIT);
  debug_assert!(additional_stages.len() <= HookTapIndex::INDEX_LIMIT);
  debug_assert!(base_stages.windows(2).all(|stages| stages[0] <= stages[1]));
  let additional_order = sort_indices_by_stage(additional_stages);
  debug_assert_eq!(additional_order.len(), additional_stages.len());
  debug_assert!(
    additional_order
      .iter()
      .all(|&index| (index as usize) < additional_stages.len())
  );
  MergedTapIndicesByStage {
    base_stages,
    additional_stages,
    additional_order,
    base_index: 0,
    additional_cursor: 0,
  }
}

impl Iterator for MergedTapIndicesByStage<'_> {
  type Item = HookTapIndex;

  fn next(&mut self) -> Option<Self::Item> {
    let base_index = self.base_index as usize;
    let additional_cursor = self.additional_cursor as usize;
    debug_assert!(base_index <= self.base_stages.len());
    debug_assert!(additional_cursor <= self.additional_order.len());
    if base_index == self.base_stages.len() && additional_cursor == self.additional_order.len() {
      return None;
    }

    if additional_cursor == self.additional_order.len() {
      let index = self.base_index;
      debug_assert!(index <= HookTapIndex::INDEX_MASK);
      self.base_index += 1;
      return Some(HookTapIndex::tap(index));
    }

    if base_index == self.base_stages.len() {
      let index = self.additional_order[additional_cursor];
      debug_assert!(self.additional_cursor <= HookTapIndex::INDEX_MASK);
      self.additional_cursor += 1;
      return Some(HookTapIndex::intercept(index));
    }

    let additional_index = self.additional_order[additional_cursor];
    if self.base_stages[base_index] <= self.additional_stages[additional_index as usize] {
      let index = self.base_index;
      debug_assert!(index <= HookTapIndex::INDEX_MASK);
      self.base_index += 1;
      Some(HookTapIndex::tap(index))
    } else {
      debug_assert!(self.additional_cursor <= HookTapIndex::INDEX_MASK);
      self.additional_cursor += 1;
      Some(HookTapIndex::intercept(additional_index))
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

  fn used_stages(&self) -> Vec<i32>;

  fn load_js_tap_register(&mut self, register: JsTapRegister) -> Result<()>;

  fn has_js_taps(&self) -> bool;

  fn is_empty(&self) -> bool;

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
  pub use tracing;
}

pub use rspack_macros::{define_hook, plugin, plugin_hook};
