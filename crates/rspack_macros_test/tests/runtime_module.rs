use std::{marker::PhantomData, sync::Arc};

use rspack_core::{rspack_sources::Source, Compilation};
use rspack_error::Result;
use rspack_identifier::Identifier;
use rspack_macros::impl_runtime_module;

#[test]
fn with_generic() {
  #[impl_runtime_module]
  #[derive(Debug)]
  struct Foo<T: std::fmt::Debug + Send + Sync + Eq + 'static> {
    marker: PhantomData<T>,
  }

  impl<T: std::fmt::Debug + Send + Sync + Eq + 'static> Foo<T> {
    fn name(&self) -> Identifier {
      String::new().into()
    }
    fn generate_with_custom(&self, _compilation: &Compilation) -> Result<Arc<dyn Source>> {
      todo!()
    }
  }
}
