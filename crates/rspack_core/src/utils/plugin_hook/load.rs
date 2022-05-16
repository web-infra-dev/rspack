use tracing::instrument;

use crate::{plugin_driver::PluginDriver, Loader};

#[instrument(skip_all)]
#[inline]
pub async fn load(id: &str, plugin_driver: &PluginDriver) -> (String, Loader) {
  let plugin_result = plugin_driver.load(id).await;
  let content = plugin_result
    .clone()
    .and_then(|load_output| load_output.content)
    .unwrap_or_else(|| std::fs::read_to_string(id).expect(&format!("load failed for {:?}", id)));
  let loader = plugin_result
    .and_then(|load_output| load_output.loader)
    .unwrap_or_else(|| panic!("No loader to deal with file: {:?}", id));
  (content, loader)
}
