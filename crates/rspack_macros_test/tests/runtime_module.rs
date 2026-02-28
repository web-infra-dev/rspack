use std::marker::PhantomData;

use rspack_core::{RuntimeModule, RuntimeModuleGenerateContext, rspack_sources::Source};
use rspack_macros::impl_runtime_module;

#[allow(dead_code)]
#[test]
fn with_generic() {
  #[impl_runtime_module]
  #[derive(Debug)]
  struct Foo<T: std::fmt::Debug + Send + Sync + Eq + 'static> {
    marker: PhantomData<T>,
  }

  #[async_trait::async_trait]
  impl<T: std::fmt::Debug + Send + Sync + Eq + 'static> RuntimeModule for Foo<T> {
    async fn generate(&self, _: &RuntimeModuleGenerateContext<'_>) -> rspack_error::Result<String> {
      todo!()
    }
  }
}
