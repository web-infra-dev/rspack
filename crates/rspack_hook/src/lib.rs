use async_trait::async_trait;
use rspack_error::Result;
use rustc_hash::FxHashSet;

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

  fn tap_stage(tap: &Self::Tap) -> i32;

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

  use crate::{Hook, Interceptor};

  pub struct HookTaps<H: Hook> {
    taps: Vec<H::Tap>,
    interceptors: Vec<Box<dyn Interceptor<H> + Send + Sync>>,
  }

  impl<H: Hook> Default for HookTaps<H> {
    fn default() -> Self {
      Self {
        taps: Default::default(),
        interceptors: Default::default(),
      }
    }
  }

  impl<H: Hook> HookTaps<H> {
    pub fn used_stages(&self) -> FxHashSet<i32> {
      FxHashSet::from_iter(self.taps.iter().map(H::tap_stage))
    }

    pub fn intercept(&mut self, interceptor: impl Interceptor<H> + Send + Sync + 'static) {
      self.interceptors.push(Box::new(interceptor));
    }

    pub fn tap(&mut self, tap: H::Tap) {
      self.taps.push(tap);
    }

    pub fn is_empty(&self) -> bool {
      self.taps.is_empty() && self.interceptors.is_empty()
    }

    pub async fn call_interceptors(&self, hook: &H) -> Result<Vec<H::Tap>> {
      let mut additional_taps = Vec::new();
      for interceptor in self.interceptors.iter() {
        additional_taps.extend(interceptor.call(hook).await?);
      }
      Ok(additional_taps)
    }

    pub fn call_interceptors_blocking(&self, hook: &H) -> Result<Vec<H::Tap>> {
      let mut additional_taps = Vec::new();
      for interceptor in self.interceptors.iter() {
        additional_taps.extend(interceptor.call_blocking(hook)?);
      }
      Ok(additional_taps)
    }

    pub fn sorted_taps<'a>(&'a self, additional_taps: &'a [H::Tap]) -> Vec<&'a H::Tap> {
      let mut all_taps = Vec::with_capacity(self.taps.len() + additional_taps.len());
      all_taps.extend(&self.taps);
      all_taps.extend(additional_taps);
      all_taps.sort_by_key(|hook| H::tap_stage(hook));
      all_taps
    }
  }
}

pub use rspack_macros::{define_hook, plugin, plugin_hook};
