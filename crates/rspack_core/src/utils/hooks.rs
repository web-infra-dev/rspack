use std::path::Path;

use crate::{NormalModuleFactoryContext, PluginDriver, ResolveArgs, ResolveResult};

pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &PluginDriver,
  _job_context: &mut NormalModuleFactoryContext,
) -> anyhow::Result<String> {
  // plugin_driver.resolver
  let base_dir = if let Some(importer) = args.importer {
    Path::new(importer)
      .parent()
      .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
  } else {
    Path::new(plugin_driver.options.context.as_str())
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
      ResolveResult::Info(info) => info.path.to_string_lossy().to_string(),
      ResolveResult::Ignored => format!("UnReachable:{}", args.specifier),
    }
  })
}
