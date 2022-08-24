use std::sync::Arc;

use rspack_error::Result;

use rspack_loader_runner::{Content, LoaderRunnerPlugin, ResourceData};

use crate::PluginDriver;

struct ProcessResource {
  plugin_driver: Arc<PluginDriver>,
}

#[async_trait::async_trait]
impl LoaderRunnerPlugin for ProcessResource {
  async fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
    let result = self.plugin_driver.read_resource(resource_data).await?;
    if result.is_some() {
      return Ok(result);
    }

    Ok(None)
  }
}
