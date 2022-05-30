use std::path::Path;

use anyhow::{Context, Result};
use tracing::instrument;

use crate::{plugin_driver::PluginDriver, LoadArgs, Loader, LoaderOptions};

#[instrument(skip_all)]
#[inline]
pub async fn load(
  args: LoadArgs,
  plugin_driver: &PluginDriver,
) -> Result<(String, Option<Loader>)> {
  let plugin_result = plugin_driver.load(&args).await?;
  let content = plugin_result
    .clone()
    .and_then(|load_output| load_output.content);

  let content = match content {
    Some(content) => Ok(content),
    None => std::fs::read_to_string(args.id.as_str())
      .with_context(|| format!("failed to load content from {}", args.id)),
  }?;

  let loader = plugin_result.map_or_else(
    || guess_loader_by_id(args.id.as_str(), &plugin_driver.ctx.options.loader),
    |load_output| load_output.loader,
  );
  Ok((content, loader))
}

fn guess_loader_by_id(id: &str, options: &LoaderOptions) -> Option<Loader> {
  let ext = if let Some(ext) = Path::new(id).extension() {
    ext.to_str()?
  } else {
    "js"
  };
  Some(*options.get(ext)?)
}
