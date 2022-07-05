use crate::{
  parse_to_url, Content, LoadArgs, NormalModuleFactoryContext, PluginDriver, ResolveArgs,
  TransformArgs,
};
use nodejs_resolver::ResolveResult;
use std::path::Path;

pub async fn load(
  plugin_driver: &PluginDriver,
  args: LoadArgs<'_>,
  job_ctx: &mut NormalModuleFactoryContext,
) -> anyhow::Result<Content> {
  let plugin_output = plugin_driver.load(args.clone(), job_ctx).await?;

  if let Some(output) = plugin_output {
    Ok(output)
  } else {
    let url = parse_to_url(args.uri);
    debug_assert_eq!(url.scheme(), "specifier");
    Ok(Content::String(
      tokio::fs::read_to_string(url.path()).await?,
    ))
  }
}

pub fn transform(_args: TransformArgs) -> String {
  todo!()
}

pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &PluginDriver,
  job_context: &mut NormalModuleFactoryContext,
) -> anyhow::Result<String> {
  // TODO: plugins

  let plugin_output = plugin_driver.resolve(args.clone(), job_context).await?;

  if let Some(output) = plugin_output {
    return Ok(output);
  }

  // plugin_driver.resolver
  let base_dir = if let Some(importer) = args.importer {
    Path::new(importer)
      .parent()
      .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
  } else {
    Path::new(plugin_driver.options.root.as_str())
  };
  Ok({
    tracing::trace!(
      "resolved importer:{:?},specifier:{:?}",
      args.importer,
      args.specifier
    );
    match plugin_driver
      .resolver
      .resolve(base_dir, args.specifier)
      .map_err(|_| {
        anyhow::format_err!(
          "fail to resolved importer:{:?},specifier:{:?}",
          args.importer,
          args.specifier
        )
      })? {
      ResolveResult::Path(buf) => buf.to_string_lossy().to_string(),
      _ => unreachable!(),
    }
  })
}
