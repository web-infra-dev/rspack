use std::path::Path;

use tracing::instrument;

use crate::{plugin_driver::PluginDriver, LoadArgs, Loader, LoaderOptions};

#[instrument(skip_all)]
#[inline]
pub async fn load(args: LoadArgs, plugin_driver: &PluginDriver) -> (String, Option<Loader>) {
  let plugin_result = plugin_driver.load(&args).await;
  let content = plugin_result
    .clone()
    .and_then(|load_output| load_output.content)
    .unwrap_or_else(|| {
      std::fs::read_to_string(args.id.as_str())
        .unwrap_or_else(|_| panic!("load failed for {:?}", args.id))
    });
  let loader = plugin_result.map_or_else(
    || guess_loader_by_id(args.id.as_str(), &plugin_driver.ctx.options.loader),
    |load_output| load_output.loader,
  );
  (content, loader)
}

fn guess_loader_by_id(id: &str, options: &LoaderOptions) -> Option<Loader> {
  let ext = if let Some(ext) = Path::new(id).extension() {
    ext.to_str()?
  } else {
    "js"
  };
  Some(*options.get(ext)?)
}
