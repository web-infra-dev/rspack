#![feature(let_chains)]

use async_trait::async_trait;
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::{DisplayWithSuffix, Identifiable, Identifier};
use serde_json::json;

pub struct SimpleLoader;
#[async_trait]
impl Loader<RunnerContext> for SimpleLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.content.take() else {
      return Ok(());
    };
    let export = format!("{}-simple", content.try_into_string()?);
    loader_context.content = Some(format!("module.exports = {}", json!(export)).into());
    Ok(())
  }
}
impl Identifiable for SimpleLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_LOADER_IDENTIFIER.into()
  }
}
pub const SIMPLE_LOADER_IDENTIFIER: &str = "builtin:test-simple-loader";

pub struct SimpleAsyncLoader;
#[async_trait]
impl Loader<RunnerContext> for SimpleAsyncLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.content.take() else {
      return Ok(());
    };
    loader_context.content = Some(format!("{}-async-simple", content.try_into_string()?).into());
    Ok(())
  }
}
impl Identifiable for SimpleAsyncLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_ASYNC_LOADER_IDENTIFIER.into()
  }
}
pub const SIMPLE_ASYNC_LOADER_IDENTIFIER: &str = "builtin:test-simple-async-loader";

pub struct PitchingLoader;
#[async_trait]
impl Loader<RunnerContext> for PitchingLoader {
  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    loader_context.content = Some(
      [
        loader_context
          .remaining_request()
          .display_with_suffix(loader_context.resource()),
        loader_context.previous_request().to_string(),
      ]
      .join(":")
      .into(),
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
