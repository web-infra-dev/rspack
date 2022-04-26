use tracing::instrument;

use crate::plugin_driver::PluginDriver;

#[instrument(skip(plugin_driver))]
#[inline]
pub async fn load(id: &str, plugin_driver: &PluginDriver) -> String {
  let plugin_result = plugin_driver.load(id).await;

  plugin_result.unwrap_or_else(|| std::fs::read_to_string(id).unwrap())
}
