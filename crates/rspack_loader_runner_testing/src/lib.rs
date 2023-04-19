#[macro_use]
extern crate napi_derive;

use async_trait::async_trait;
use rspack_core::{Loader, LoaderContext, LoaderRunnerContext};
use rspack_error::Result;
use rspack_identifier::{Identifiable, Identifier};

#[napi]
pub struct JsLoader {
  pub ident: Identifier,
}

#[async_trait]
impl Loader<LoaderRunnerContext> for JsLoader {
  async fn run(&self, _loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    // noop
    Ok(())
  }

  async fn pitch(
    &self,
    _loader_context: &mut LoaderContext<'_, LoaderRunnerContext>,
  ) -> Result<()> {
    // noop
    Ok(())
  }
}

impl Identifiable for JsLoader {
  fn identifier(&self) -> Identifier {
    self.ident
  }
}
