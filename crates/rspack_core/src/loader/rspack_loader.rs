use std::sync::{Arc, Mutex};

use rspack_error::{Diagnostic, Result};
use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::{Content, LoaderContext, LoaderRunnerPlugin, ResourceData};
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

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
  ) -> Result<Option<(Content, Option<SourceMap>, HashSet<std::path::PathBuf>)>> {
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
            // Convert file dependencies to FxHashSet for consistency
            let file_deps = extract_result
              .file_dependencies
              .map(|deps| deps.into_iter().collect::<HashSet<_>>())
              .unwrap_or_default();

            // Return the content with source map extracted and file dependencies
            return Ok(Some((
              Content::String(extract_result.source),
              extract_result.source_map,
              file_deps,
            )));
          }
          Err(e) => {
            // If extraction fails, return original content with empty dependencies
            // Log the error as a warning
            self
              .plugin_driver
              .diagnostics
              .lock()
              .expect("should get lock")
              .push(Diagnostic::warn("extractSourceMap".into(), e));
            return Ok(Some((content, None, HashSet::default())));
          }
        }
      }
      // Return original content with empty dependencies when extract_source_map is disabled
      return Ok(Some((content, None, HashSet::default())));
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
