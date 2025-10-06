use std::sync::{Arc, Mutex};

use rspack_error::{Diagnostic, Result};
use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::{Content, LoaderContext, LoaderRunnerPlugin, ResourceData};
use rspack_sources::SourceMap;

use crate::{RunnerContext, SharedPluginDriver, utils::extract_source_map};

pub struct RspackLoaderRunnerPlugin {
  pub plugin_driver: SharedPluginDriver,
  pub current_loader: Mutex<Option<String>>,
  pub extract_source_map: Option<bool>,
}

#[async_trait::async_trait]
impl LoaderRunnerPlugin for RspackLoaderRunnerPlugin {
  type Context = RunnerContext;

  fn name(&self) -> &'static str {
    "rspack-loader-runner"
  }

  async fn before_all(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    self
      .plugin_driver
      .normal_module_hooks
      .loader
      .call(context)
      .await
  }

  async fn process_resource(
    &self,
    resource_data: &ResourceData,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Option<(Content, Option<SourceMap>)>> {
    // First try the plugin's read_resource hook
    let result = self
      .plugin_driver
      .normal_module_hooks
      .read_resource
      .call(resource_data, &fs)
      .await?;

    if let Some(content) = result {
      if let Some(true) = self.extract_source_map {
        // Try to extract source map from the content
        let extract_result = match &content {
          Content::String(s) => extract_source_map(fs, s, resource_data.resource()).await,
          Content::Buffer(b) => {
            extract_source_map(fs, &String::from_utf8_lossy(b), resource_data.resource()).await
          }
        };

        match extract_result {
          Ok(extract_result) => {
            // Return the content with source map extracted
            // The source map will be available through the loader context
            return Ok(Some((
              Content::String(extract_result.source),
              extract_result.source_map,
            )));
          }
          Err(e) => {
            // If extraction fails, return original content
            // Log the error as a warning
            self
              .plugin_driver
              .diagnostics
              .lock()
              .expect("should get lock")
              .push(Diagnostic::warn("extractSourceMap".into(), e));
            return Ok(Some((content, None)));
          }
        }
      }
      return Ok(Some((content, None)));
    }

    // If no plugin handled it, return None so the default logic can handle it
    Ok(None)
  }

  async fn should_yield(&self, context: &LoaderContext<Self::Context>) -> Result<bool> {
    let res = self
      .plugin_driver
      .normal_module_hooks
      .loader_should_yield
      .call(context)
      .await?;

    if let Some(res) = res {
      return Ok(res);
    }

    Ok(false)
  }

  async fn start_yielding(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    self
      .plugin_driver
      .normal_module_hooks
      .loader_yield
      .call(context)
      .await
  }
}
