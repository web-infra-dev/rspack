use std::{marker::PhantomData, sync::Arc};

use rspack_core_macros::impl_runtime_module;
use rspack_identifier::Identifier;
use rspack_sources::Source;

#[test]
#[allow(unused)]
fn macro_should_compile() {
  #[impl_runtime_module]
  #[derive(Debug, Eq)]
  struct Foo<T: std::fmt::Debug + Send + Sync + Eq + 'static> {
    marker: PhantomData<T>,
  }

  impl<T: std::fmt::Debug + Send + Sync + Eq + 'static> Foo<T> {
    fn name(&self) -> Identifier {
      String::new().into()
    }
    fn as_str(&self) {}
    fn generate_with_custom(&self, _compilation: &rspack_core::Compilation) -> Arc<dyn Source> {
      todo!()
    }
  }
}
