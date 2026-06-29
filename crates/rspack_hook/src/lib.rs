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
  use std::marker::PhantomData;

  pub use async_trait::async_trait;
  pub use rspack_error::Result;
  pub use rustc_hash::FxHashSet;
  pub use tracing;

  pub trait PluginHookTapPlugin: Sized {
    fn clone_for_plugin_hook_tap(&self) -> Self;
  }

  #[allow(non_camel_case_types)]
  pub struct PluginHookTap<M, P> {
    pub plugin: P,
    marker: PhantomData<fn() -> M>,
  }

  impl<M, P> PluginHookTap<M, P>
  where
    P: PluginHookTapPlugin,
  {
    pub fn new(plugin: &P) -> Self {
      Self {
        plugin: plugin.clone_for_plugin_hook_tap(),
        marker: PhantomData,
      }
    }
  }
}

pub use rspack_macros::{define_hook, plugin, plugin_hook};
