mod create_tree_code_from_path;
mod load_entrypoint;
mod options;

use std::borrow::Cow;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier};

pub use crate::options::Options;

pub const NEXT_APP_LOADER_IDENTIFIER: &str = "builtin:next-app-loader";

#[cacheable]
#[derive(Debug)]
pub struct NextAppLoader {
  id: Identifier,
  options: Options,
}

impl NextAppLoader {
  pub fn new(options: Options, ident: &str) -> Self {
    Self {
      id: ident.into(),
      options,
    }
  }

  async fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(resource_path) = loader_context.resource_path() else {
      return Ok(());
    };

    let filename = resource_path.as_str().to_string();

    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let content_str = match &content {
      rspack_core::Content::String(s) => Cow::Borrowed(s.as_str()),
      rspack_core::Content::Buffer(buf) => String::from_utf8_lossy(buf),
    };

    loader_context.finish_with("".to_string());

    Ok(())
  }
}

impl Identifiable for NextAppLoader {
  fn identifier(&self) -> rspack_loader_runner::Identifier {
    self.id
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for NextAppLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    // for better diagnostic, as async_trait macro don't show beautiful error message
    self.loader_impl(loader_context).await
  }
}
