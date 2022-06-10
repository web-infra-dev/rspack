use crate::{JobContext, LoadArgs, PluginDriver, ResolveArgs, TransformArgs};
use nodejs_resolver::ResolveResult;
use std::path::Path;
use sugar_path::PathSugar;

pub async fn load(args: LoadArgs<'_>) -> anyhow::Result<String> {
  Ok(tokio::fs::read_to_string(args.uri).await?)
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

  let resolved = if let Some(importer) = args.importer {
    let base_dir = Path::new(importer)
      .parent()
      .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?;

    tracing::trace!(
      "resolved importer:{:?},specifier:{:?}",
      importer,
      args.specifier
    );
    match plugin_driver
      .resolver
      .resolve(base_dir, args.specifier)
      .map_err(|_| {
        anyhow::format_err!(
          "fail to resolved importer:{:?},specifier:{:?}",
          importer,
          args.specifier
        )
      })? {
      ResolveResult::Path(buf) => buf.to_string_lossy().to_string(),
      _ => unreachable!(),
    }
  } else {
    Path::new(plugin_driver.options.root.as_str())
      .join(&args.specifier)
      .resolve()
      .to_string_lossy()
      .to_string()
  };
  Ok(resolved)
}
