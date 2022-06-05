use std::path::Path;

use anyhow::{Context, Result};
use tracing::instrument;

use crate::{task::TaskContext, LoadArgs, Loader, LoaderOptions, PluginDriver};

#[instrument(skip_all)]
#[inline]
pub async fn load(
  args: LoadArgs,
  plugin_driver: &PluginDriver,
  task_context: &mut TaskContext,
) -> Result<(String, Option<Loader>)> {
  let plugin_result = plugin_driver.load(&args, task_context).await?;
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
  let loader = if let Some(ext) = Path::new(id).extension() {
    *options.get(ext.to_str()?)?
  } else {
    Loader::Js
  };
  Some(loader)
}
