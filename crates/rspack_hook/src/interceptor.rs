use async_trait::async_trait;
use rspack_error::Result;

#[async_trait]
pub trait Interceptor<H: Hook> {
  async fn call(&self, hook: &H) -> Result<Vec<<H as Hook>::Tap>>;
}

pub trait Hook {
  type Tap;
}
