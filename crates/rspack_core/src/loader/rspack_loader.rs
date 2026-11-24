use std::sync::Arc;

use rspack_error::{Diagnostic, Result};
use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::{
  Content, LoaderContext, LoaderDependencies, LoaderRunnerPlugin, ResourceData,
};
use rspack_paths::{InternedPath, Utf8Path};
use rspack_sources::SourceMap;

use crate::{
  RunnerContext, SharedPluginDriver,
  loader::loader_cache::{LoaderCacheAction, after_normal_chain, before_normal_chain},
  utils::extract_source_map,
};

pub struct RspackLoaderRunnerPlugin {
  pub plugin_driver: SharedPluginDriver,
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
    diagnostics: &mut Vec<Diagnostic>,
    dependencies: &mut LoaderDependencies,
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
        let resource_path = resource_data
          .path()
          .map_or_else(|| resource_data.resource(), Utf8Path::as_str);
        let extract_result = match &content {
          Content::String(s) => extract_source_map(fs, s, resource_path).await,
          Content::Buffer(b) => match simdutf8::basic::from_utf8(b) {
            Ok(s) => extract_source_map(fs, s, resource_path).await,
            Err(_) => extract_source_map(fs, &String::from_utf8_lossy(b), resource_path).await,
          },
        };

        match extract_result {
          Ok(extract_result) => {
            if let Some(file_dependencies) = extract_result.file_dependencies {
              dependencies
                .file
                .extend(file_dependencies.into_iter().map(InternedPath::from));
            }

            // Preserve the input type: non-raw JS loaders strip a BOM only from buffers.
            let content = if content.is_buffer() {
              Content::Buffer(extract_result.source.into_bytes())
            } else {
              Content::String(extract_result.source)
            };

            return Ok(Some((content, extract_result.source_map)));
          }
          Err(e) => diagnostics.push(Diagnostic::warn("extractSourceMap".into(), e)),
        }
      }
      return Ok(Some((content, None)));
    }

    // If no plugin handled it, return None so the default logic can handle it
    Ok(None)
  }

  async fn start_yielding(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    self
      .plugin_driver
      .normal_module_hooks
      .loader_yield
      .call(context)
      .await
  }

  async fn run_normal_chain(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    let chain = context
      .current_root_chain()
      .expect("normal execution requires a current root chain");
    let range = chain.range();
    if chain.is_cache() && context.loader_index == chain.end() as i32 - 1 {
      let loader_index = context.loader_index as usize;
      if context.loader_items[loader_index].filter().is_some()
        && context.loader_items[loader_index].should_skip_normal(context.content())
      {
        context.loader_items[loader_index].set_normal_executed();
        context.loader_index -= 1;
        return Ok(());
      }
    }
    let cache_action = if chain.is_cache() && context.loader_index == chain.end() as i32 - 1 {
      before_normal_chain(context).await?
    } else {
      LoaderCacheAction::Disabled
    };
    if matches!(cache_action, LoaderCacheAction::Hit) {
      for loader_index in range.clone() {
        let state = &context.loader_items[usize::from(loader_index)];
        state.set_normal_executed();
        state.set_finish_called();
      }
      context.loader_index = i32::from(range.start) - 1;
      context.merge_dependency_changes();
      return Ok(());
    }

    context.run_normal_chain().await?;
    if let LoaderCacheAction::Miss(state) = cache_action {
      after_normal_chain(context, &state).await;
    }
    Ok(())
  }
}
