use std::path::Path;

use tracing::instrument;

use crate::{plugin_driver::PluginDriver, Loader, LoaderOptions};

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
    .unwrap_or_else(|| {
      guess_loader_by_id(id, &plugin_driver.ctx.options.loader)
        .unwrap_or_else(|| panic!("No loader to deal with file: {:?}", id))
    });
  (content, loader)
}

fn guess_loader_by_id(id: &str, options: &LoaderOptions) -> Option<Loader> {
  let loader = *Path::new(id)
    .extension()
    .and_then(|ext| ext.to_str())
    .and_then(|ext| options.get(ext))?;

  Some(loader)
}
