use rspack_error::Result;
use rspack_loader_runner::{Content, LoaderContext, LoaderRunnerPlugin, ResourceData};

use crate::{CompilerContext, NormalModule, SharedPluginDriver};

pub struct RspackLoaderRunnerPlugin<'a> {
  pub plugin_driver: SharedPluginDriver,
  pub normal_module: &'a NormalModule,
}

impl<'a> RspackLoaderRunnerPlugin<'a> {
  pub fn new(plugin_driver: SharedPluginDriver, normal_module: &'a NormalModule) -> Self {
    Self {
      plugin_driver,
      normal_module,
    }
  }
}

#[async_trait::async_trait]
impl LoaderRunnerPlugin for RspackLoaderRunnerPlugin<'_> {
  type Context = CompilerContext;

  fn name(&self) -> &'static str {
    "rspack-loader-runner"
  }

  fn loader_context(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    self
      .plugin_driver
      .normal_module_loader(context, self.normal_module)
  }

  async fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
    let result = self.plugin_driver.read_resource(resource_data).await?;
    if result.is_some() {
      return Ok(result);
    }

    Ok(None)
  }
}
