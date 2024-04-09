use std::sync::Mutex;

use rspack_error::Result;
use rspack_loader_runner::{Content, LoaderContext, LoaderRunnerPlugin, ResourceData};

use crate::{CompilerContext, SharedPluginDriver};

pub struct RspackLoaderRunnerPlugin {
  pub plugin_driver: SharedPluginDriver,
  pub current_loader: Mutex<Option<String>>,
}

#[async_trait::async_trait]
impl LoaderRunnerPlugin for RspackLoaderRunnerPlugin {
  type Context = CompilerContext;

  fn name(&self) -> &'static str {
    "rspack-loader-runner"
  }

  fn loader_context(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    self
      .plugin_driver
      .normal_module_hooks
      .loader
      .call(&mut context.hot)
  }

  fn before_each(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    *self.current_loader.lock().expect("failed to lock") =
      Some(context.current_loader().to_string());
    Ok(())
  }

  async fn process_resource(&self, resource_data: &mut ResourceData) -> Result<Option<Content>> {
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
}
