use std::path::Path;

use tracing::instrument;

use crate::{plugin_driver::PluginDriver, Loader, LoaderOptions};

#[instrument(skip_all)]
#[inline]
pub async fn load(id: &str, plugin_driver: &PluginDriver) -> (String, Option<Loader>) {
  let query_start = id.find(|c: char| c == '?').or(Some(id.len())).unwrap();
  let file_path = &id[..query_start];
  let plugin_result = plugin_driver.load(id).await;
  let content = plugin_result
    .clone()
    .and_then(|load_output| load_output.content)
    .unwrap_or_else(|| {
      std::fs::read_to_string(file_path).expect(&format!("load failed for {:?}", id))
    });
  let loader = plugin_result.map_or_else(
    || guess_loader_by_id(file_path, &plugin_driver.ctx.options.loader),
    |load_output| load_output.loader,
  );
  (content, loader)
}

fn guess_loader_by_id(id: &str, options: &LoaderOptions) -> Option<Loader> {
  let loader = *Path::new(id)
    .extension()
    .and_then(|ext| ext.to_str())
    .and_then(|ext| options.get(ext))?;

  Some(loader)
}
