use crate::{ResolveArgs, ResolveResult, SharedPluginDriver};
use rspack_error::{internal_error, Error, Result, TraceableError};
use std::{path::Path, sync::Arc};
use tracing::instrument;

#[instrument(name = "resolve", skip_all)]
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
  //  _job_context: &mut NormalModuleFactoryContext,
) -> Result<ResolveResult> {
  let plugin_driver = plugin_driver.read().await;
  let base_dir = if let Some(importer) = args.importer {
    {
      // TODO: delete this fn after use `normalModule.context` rather than `importer`
      if let Some(index) = importer.find('?') {
        Path::new(&importer[0..index])
      } else {
        Path::new(importer)
      }
    }
    .parent()
    .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
  } else {
    &plugin_driver.options.context
  };
  tracing::trace!(
    "resolved importer:{:?},specifier:{:?}",
    args.importer,
    args.specifier
  );
  let resolver = args
    .resolve_options
    .map(|resolve_options| Arc::new(plugin_driver.resolver_factory.get(resolve_options)))
    .unwrap_or_else(|| Arc::clone(&plugin_driver.resolver));
  resolver
    .resolve(base_dir, args.specifier)
    .map_err(|error| match error {
      nodejs_resolver::Error::Io(error) => Error::Io { source: error },
      nodejs_resolver::Error::UnexpectedJson((json_path, error)) => Error::Anyhow {
        source: anyhow::Error::msg(format!("{:?} in {:?}", error, json_path)),
      },
      nodejs_resolver::Error::UnexpectedValue(error) => Error::Anyhow {
        source: anyhow::Error::msg(error),
      },
      _ => {
        if let Some(importer) = args.importer {
          let span = args.span.unwrap_or_default();
          let message = if let nodejs_resolver::Error::Overflow = error {
            format!(
              "Can't resolve {:?} in {importer} , maybe it had cycle alias",
              args.specifier,
            )
          } else {
            format!("Failed to resolve {} in {importer}", args.specifier)
          };
          Error::TraceableError(TraceableError::from_path(
            importer.to_string(),
            span.start as usize,
            span.end as usize,
            "Resolve error".to_string(),
            message,
          ))
        } else {
          Error::InternalError(internal_error!(format!(
            "Failed to resolve {} in {}",
            args.specifier,
            plugin_driver.options.context.display()
          )))
        }
      }
    })
}
