use crate::plugin_driver::PluginDriver;

#[inline]
pub async fn load(id: &str, plugin_dirver: &PluginDriver) -> String {
    let plugin_result = plugin_dirver.load(id).await;

    plugin_result.unwrap_or_else(|| std::fs::read_to_string(id).unwrap())
}
