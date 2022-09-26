use std::sync::Arc;

use rspack_error::Result;

use rspack_loader_runner::{Content, LoaderRunnerPlugin, ResourceData};

use crate::PluginDriver;

pub struct LoaderRunnerPluginProcessResource {
  plugin_driver: Arc<PluginDriver>,
}

impl LoaderRunnerPluginProcessResource {
  pub fn new(plugin_driver: Arc<PluginDriver>) -> Self {
    Self { plugin_driver }
  }
}

impl LoaderRunnerPlugin for LoaderRunnerPluginProcessResource {
  fn name(&self) -> &'static str {
    "process-resource"
  }

  fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
    let result = self.plugin_driver.read_resource(resource_data)?;
    if result.is_some() {
      return Ok(result);
    }

    Ok(None)
  }
}
