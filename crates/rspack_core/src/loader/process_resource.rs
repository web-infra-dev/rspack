use rspack_error::Result;
use rspack_loader_runner2::{Content, LoaderRunnerPlugin, ResourceData};

use crate::SharedPluginDriver;

pub struct LoaderRunnerPluginProcessResource {
  plugin_driver: SharedPluginDriver,
}

impl LoaderRunnerPluginProcessResource {
  pub fn new(plugin_driver: SharedPluginDriver) -> Self {
    Self { plugin_driver }
  }
}

#[async_trait::async_trait]
impl LoaderRunnerPlugin for LoaderRunnerPluginProcessResource {
  fn name(&self) -> &'static str {
    "process-resource"
  }

  async fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
    let result = self
      .plugin_driver
      .read()
      .await
      .read_resource(resource_data)
      .await?;
    if result.is_some() {
      return Ok(result);
    }

    Ok(None)
  }
}
