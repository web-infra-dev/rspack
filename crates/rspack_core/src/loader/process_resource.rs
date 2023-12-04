use rspack_error::{internal_error, Result};
use rspack_loader_runner::{Content, LoaderRunnerPlugin, ResourceData};

use crate::SharedPluginDriver;

pub struct LoaderRunnerPluginProcessResource {
  pub plugin_driver: SharedPluginDriver,
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
    let result = self.plugin_driver.read_resource(resource_data).await?;
    if result.is_some() {
      return Ok(result);
    }

    Err(internal_error!(
      r#"Reading from "{}" is not handled by plugins (Unhandled scheme).
Rspack supports "data:" and "file:" URIs by default.
You may need an additional plugin to handle "{}:" URIs."#,
      resource_data.resource,
      resource_data.get_scheme()
    ))
  }
}
