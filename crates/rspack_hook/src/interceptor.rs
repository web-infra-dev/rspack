use async_trait::async_trait;
use rspack_error::Result;

#[async_trait]
pub trait Interceptor<Hook> {
  async fn call(&self, hook: &Hook) -> Result<()>;
}
