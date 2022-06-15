use crate::{parse_to_url, JobContext, LoadArgs, PluginDriver, ResolveArgs, TransformArgs};
use nodejs_resolver::ResolveResult;
use std::path::Path;
use sugar_path::PathSugar;

pub async fn load(args: LoadArgs<'_>) -> anyhow::Result<String> {
  let url = parse_to_url(args.uri);
  assert_eq!(url.scheme(), "specifier");
  Ok(tokio::fs::read_to_string(url.path()).await?)
}

pub fn transform(_args: TransformArgs) -> String {
  todo!()
}

pub fn resolve(
  args: ResolveArgs,
  plugin_driver: &PluginDriver,
  _job_context: &mut JobContext,
) -> anyhow::Result<String> {
  // TODO: plugins

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
