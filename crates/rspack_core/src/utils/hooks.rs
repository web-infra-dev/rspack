use crate::{LoadArgs, PluginDriver, ResolveArgs, TransformArgs};
use nodejs_resolver::ResolveResult;
use std::path::Path;
use sugar_path::PathSugar;

pub async fn load(args: LoadArgs<'_>) -> String {
  tokio::fs::read_to_string(args.uri)
    .await
    .unwrap_or_else(|_| panic!("unable to read from {:?}", args.uri))
}

pub fn transform(_args: TransformArgs) -> String {
  todo!()
}

pub fn resolve(args: ResolveArgs, plugin_driver: &PluginDriver) -> String {
  // TODO: plugins

  // plugin_driver.resolver

  if let Some(importer) = args.importer {
    let base_dir = Path::new(importer).parent().unwrap();
    tracing::trace!(
      "resolved importer:{:?},specifier:{:?}",
      importer,
      args.specifier
    );
    match plugin_driver
      .resolver
      .resolve(base_dir, args.specifier)
      .unwrap_or_else(|_| {
        panic!(
          "fail to resolved importer:{:?},specifier:{:?}",
          importer, args.specifier
        )
      }) {
      ResolveResult::Path(buf) => buf.to_string_lossy().to_string(),
      _ => unreachable!(),
    }
  } else {
    Path::new(plugin_driver.options.root.as_str())
      .join(&args.specifier)
      .resolve()
      .to_string_lossy()
      .to_string()
  }
}
