use std::sync::Mutex;

use rspack_error::Result;
use rspack_loader_runner::{Content, LoaderContext, LoaderRunnerPlugin, ResourceData};

use crate::{RunnerContext, SharedPluginDriver};

pub struct RspackLoaderRunnerPlugin {
  pub plugin_driver: SharedPluginDriver,
  pub current_loader: Mutex<Option<String>>,
}

#[async_trait::async_trait]
impl LoaderRunnerPlugin for RspackLoaderRunnerPlugin {
  type Context = RunnerContext;

  fn name(&self) -> &'static str {
    "rspack-loader-runner"
  }

  fn before_all(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    self.plugin_driver.normal_module_hooks.loader.call(context)
  }

  async fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
    let result = self
      .plugin_driver
      .normal_module_hooks
      .read_resource
      .call(resource_data)
      .await?;
    if result.is_some() {
      return Ok(result);
    }

    Ok(None)
  }

  fn should_yield(&self, context: &LoaderContext<Self::Context>) -> Result<bool> {
    let res = self
      .plugin_driver
      .normal_module_hooks
      .loader_should_yield
      .call(context)?;

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
