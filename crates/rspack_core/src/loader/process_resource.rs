use rspack_error::Result;
use rspack_loader_runner::{Content, LoaderRunnerPlugin, ResourceData};

use crate::{NormalModuleReadResourceForSchemeArgs, SharedPluginDriver};

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
    let scheme = url::Url::parse(resource_data.resource.as_str())
      .map(|url| url.scheme().to_string())
      .unwrap_or_default();
    let result = self
      .plugin_driver
      .read()
      .await
      .read_resource(NormalModuleReadResourceForSchemeArgs {
        resource: resource_data.clone(),
        scheme,
      })
      .await?;
    if result.is_some() {
      return Ok(result);
    }

    Ok(None)
  }
}
