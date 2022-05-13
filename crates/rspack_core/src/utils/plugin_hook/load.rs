use std::path::Path;

use tracing::instrument;

use crate::{plugin_driver::PluginDriver, LoadedFile, Loader, ResolvedLoadedFile};

#[instrument(skip_all)]
#[inline]
pub async fn load(id: &str, plugin_driver: &PluginDriver) -> ResolvedLoadedFile {
  let loaded_file = {
    let plugin_result = plugin_driver.load(id).await;
    if let Some(result) = plugin_result {
      result
    } else {
      LoadedFile::new(
        tokio::fs::read_to_string(id)
          .await
          .expect(&format!("load failed for {:?}", id)),
      )
    }
  };
  ResolvedLoadedFile {
    content: loaded_file.content,
    loader: loaded_file.loader.unwrap_or_else(|| {
      let ext = Path::new(id)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("__unkonwn");
      let loader = plugin_driver
        .ctx
        .options
        .loader
        .get(ext)
        .map(|loader| *loader)
        .unwrap_or_else(|| match ext {
          "js" => Loader::Js,
          "jsx" => Loader::Jsx,
          "ts" => Loader::Ts,
          "tsx" => Loader::Tsx,
          "json" => Loader::Json,
          _ => panic!("No loader to process file {:?}", id),
        });
      loader
    }),
  }
}
