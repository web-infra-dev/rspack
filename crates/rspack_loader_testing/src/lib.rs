use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Loader, LoaderContext, RunnerContext};
use rspack_error::Result;
use rspack_loader_runner::{DisplayWithSuffix, Identifier};
use serde_json::json;

#[cacheable]
pub struct SimpleLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for SimpleLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    let export = format!("{}-simple", content.try_into_string()?);
    loader_context.finish_with(format!("module.exports = {}", json!(export)));
    Ok(())
  }
}
pub const SIMPLE_LOADER_IDENTIFIER: &str = "builtin:test-simple-loader";

#[cacheable]
pub struct SimpleAsyncLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for SimpleAsyncLoader {
  fn identifier(&self) -> Identifier {
    SIMPLE_ASYNC_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    loader_context.finish_with(format!("{}-async-simple", content.try_into_string()?));
    Ok(())
  }
}
pub const SIMPLE_ASYNC_LOADER_IDENTIFIER: &str = "builtin:test-simple-async-loader";

#[cacheable]
pub struct PitchingLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for PitchingLoader {
  fn identifier(&self) -> Identifier {
    PITCHING_LOADER_IDENTIFIER.into()
  }

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
pub const PITCHING_LOADER_IDENTIFIER: &str = "builtin:test-pitching-loader";

#[cacheable]
pub struct PassthroughLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for PassthroughLoader {
  fn identifier(&self) -> Identifier {
    PASS_THROUGH_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let patch_data = loader_context.take_all();
    loader_context.finish_with(patch_data);
    Ok(())
  }
}
pub const PASS_THROUGH_LOADER_IDENTIFIER: &str = "builtin:test-passthrough-loader";

#[cacheable]
pub struct NoPassthroughLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for NoPassthroughLoader {
  fn identifier(&self) -> Identifier {
    NO_PASS_THROUGH_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let (content, _, _) = loader_context.take_all();
    loader_context.finish_with(content);
    Ok(())
  }
}
pub const NO_PASS_THROUGH_LOADER_IDENTIFIER: &str = "builtin:test-no-passthrough-loader";

static DEPENDENCY_LOADER_RUNS: AtomicUsize = AtomicUsize::new(0);

#[cacheable]
pub struct DependencyLoader;
#[cacheable_dyn]
#[async_trait]
impl Loader<RunnerContext> for DependencyLoader {
  fn identifier(&self) -> Identifier {
    DEPENDENCY_LOADER_IDENTIFIER.into()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let dependency = loader_context
      .resource_path()
      .expect("test dependency loader requires a resource path")
      .with_file_name("overlap-dependency.txt");
    let transient_dependency = dependency.with_file_name("transient-dependency.txt");
    loader_context.add_file_dependency(transient_dependency.clone());
    assert!(
      !loader_context
        .existing_dependencies()
        .file
        .iter()
        .any(|item| item.as_path() == transient_dependency)
    );
    assert!(
      loader_context
        .dependencies()
        .file
        .iter()
        .any(|item| item.as_path() == transient_dependency)
    );
    assert!(
      loader_context
        .file_dependencies()
        .iter()
        .any(|item| item.as_path() == transient_dependency)
    );
    loader_context.remove_file_dependency(transient_dependency.clone());
    assert!(
      !loader_context
        .file_dependencies()
        .iter()
        .any(|item| item.as_path() == transient_dependency)
    );
    loader_context.add_file_dependency(dependency.clone());
    assert!(
      loader_context
        .file_dependencies()
        .iter()
        .any(|item| item.as_path() == dependency)
    );
    let value = loader_context
      .context
      .fs
      .read_to_string(&dependency)
      .await?;
    let runs = DEPENDENCY_LOADER_RUNS.fetch_add(1, Ordering::Relaxed) + 1;
    loader_context.finish_with(format!(
      "module.exports = {{ value: {}, runs: {runs} }};",
      json!(value.trim())
    ));
    Ok(())
  }
}
pub const DEPENDENCY_LOADER_IDENTIFIER: &str = "builtin:test-dependency-loader";
