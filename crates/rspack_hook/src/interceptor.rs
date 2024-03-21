use async_trait::async_trait;
use rspack_error::Result;
use rustc_hash::FxHashSet;

#[async_trait]
pub trait Interceptor<H: Hook> {
  async fn call(&self, hook: &H) -> Result<Vec<<H as Hook>::Tap>>;
}

pub trait InterceptorExt<H: Hook> {
  fn boxed(self) -> Box<dyn Interceptor<H>>;
}

impl<H: Hook, T: Interceptor<H> + 'static> InterceptorExt<H> for T {
  fn boxed(self) -> Box<dyn Interceptor<H>> {
    Box::new(self)
  }
}

pub trait Hook {
  type Tap;

  fn used_stages(&self) -> FxHashSet<i32>;

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static)
  where
    Self: Sized;
}
