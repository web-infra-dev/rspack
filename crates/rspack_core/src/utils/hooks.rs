use crate::{NormalModuleFactoryContext, ResolveArgs, ResolveResult, SharedPluginDriver};
use rspack_error::{Error, Result, TraceableError};
use std::path::Path;
use tracing::instrument;

#[instrument(name = "resolve")]
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
  _job_context: &mut NormalModuleFactoryContext,
) -> Result<ResolveResult> {
  let plugin_driver = plugin_driver.read().await;
  let base_dir = if let Some(importer) = args.importer {
    Path::new(importer)
      .parent()
      .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
  } else {
    Path::new(plugin_driver.options.context.as_str())
  };
  tracing::trace!(
    "resolved importer:{:?},specifier:{:?}",
    args.importer,
    args.specifier
  );
  plugin_driver
    .resolver
    .resolve(base_dir, args.specifier)
    .map_err(|error| {
      let is_overflow = matches!(error, Error::InternalError(_));
      let is_failed_tag = matches!(error, Error::BatchErrors(_));
      if !is_failed_tag && !is_overflow {
        error
      } else if let Some(importer) = args.importer {
        let span = args.span.unwrap_or_default();
        let message = if is_overflow {
          format!(
            "Can't resolve {:?}, maybe it had cycle alias",
            args.specifier
          )
        } else {
          format!("Failed to resolve {}", args.specifier)
        };
        Error::TraceableError(TraceableError::from_path(
          importer.to_string(),
          span.start as usize,
          span.end as usize,
          "Resolve error".to_string(),
          message,
        ))
      } else {
        error
      }
    })
}
