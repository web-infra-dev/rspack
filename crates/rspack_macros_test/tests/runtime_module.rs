use std::marker::PhantomData;

use rspack_collections::Identifier;
use rspack_core::{rspack_sources::Source, Compilation, RuntimeModule};
use rspack_macros::impl_runtime_module;

#[allow(dead_code)]
#[test]
fn with_generic() {
  #[impl_runtime_module]
  #[derive(Debug)]
  struct Foo<T: std::fmt::Debug + Send + Sync + Eq + 'static> {
    marker: PhantomData<T>,
  }

  impl<T: std::fmt::Debug + Send + Sync + Eq + 'static> RuntimeModule for Foo<T> {
    fn name(&self) -> Identifier {
      todo!()
    }

    fn generate(
      &self,
      _: &Compilation,
    ) -> rspack_error::Result<rspack_core::rspack_sources::BoxSource> {
      todo!()
    }
  }
}
