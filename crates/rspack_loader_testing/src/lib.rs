#![feature(let_chains)]

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::{DisplayWithSuffix, Identifiable, Identifier};
use serde_json::json;

#[cacheable]
pub struct SimpleLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for SimpleLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    let export = format!("{}-simple", content.try_into_string()?);
    loader_context.finish_with(format!("module.exports = {}", json!(export)));
    Ok(())
  }
}
impl Identifiable for SimpleLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_LOADER_IDENTIFIER.into()
  }
}
pub const SIMPLE_LOADER_IDENTIFIER: &str = "builtin:test-simple-loader";

#[cacheable]
pub struct SimpleAsyncLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for SimpleAsyncLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    loader_context.finish_with(format!("{}-async-simple", content.try_into_string()?));
    Ok(())
  }
}
impl Identifiable for SimpleAsyncLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_ASYNC_LOADER_IDENTIFIER.into()
  }
}
pub const SIMPLE_ASYNC_LOADER_IDENTIFIER: &str = "builtin:test-simple-async-loader";

#[cacheable]
pub struct PitchingLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for PitchingLoader {
  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    loader_context.finish_with(
      [
        loader_context
          .remaining_request()
          .display_with_suffix(loader_context.resource()),
        loader_context.previous_request().to_string(),
      ]
      .join(":"),
    );
    Ok(())
  }
}
impl Identifiable for PitchingLoader {
  fn identifier(&self) -> Identifier {
    PITCHING_LOADER_IDENTIFIER.into()
  }
}
pub const PITCHING_LOADER_IDENTIFIER: &str = "builtin:test-pitching-loader";

#[cacheable]
pub struct PassthroughLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for PassthroughLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let patch_data = loader_context.take_all();
    loader_context.finish_with(patch_data);
    Ok(())
  }
}
impl Identifiable for PassthroughLoader {
  fn identifier(&self) -> Identifier {
    PASS_THROUGH_LOADER_IDENTIFIER.into()
  }
}
pub const PASS_THROUGH_LOADER_IDENTIFIER: &str = "builtin:test-passthrough-loader";

#[cacheable]
pub struct NoPassthroughLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for NoPassthroughLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let (content, _, _) = loader_context.take_all();
    loader_context.finish_with(content);
    Ok(())
  }
}
impl Identifiable for NoPassthroughLoader {
  fn identifier(&self) -> Identifier {
    NO_PASS_THROUGH_LOADER_IDENTIFIER.into()
  }
}
pub const NO_PASS_THROUGH_LOADER_IDENTIFIER: &str = "builtin:test-no-passthrough-loader";
