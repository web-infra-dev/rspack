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
  pub use futures_concurrency;
  pub use rspack_error::Result;
  pub use rustc_hash::FxHashSet;
}

pub use rspack_macros::{define_hook, plugin, plugin_hook};
