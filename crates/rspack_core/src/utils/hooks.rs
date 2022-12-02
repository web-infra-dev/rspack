use crate::{NormalModuleFactoryContext, ResolveArgs, ResolveResult, SharedPluginDriver};
use rspack_error::{Error, Result, TraceableError};
use std::path::Path;
use tracing::instrument;

#[instrument(name = "resolve", skip_all)]
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
    &plugin_driver.options.context
  };
  tracing::trace!(
    "resolved importer:{:?},specifier:{:?}",
    args.importer,
    args.specifier
  );

  let importer = args.importer.map(|x| x.to_owned());
  let specifier = args.specifier.to_owned();
  let resolver = plugin_driver.resolver.clone();
  let context = plugin_driver.options.context.clone();
  let base_dir = base_dir.to_owned();

  tokio::task::spawn_blocking(move || {
    resolver
      .resolve(&base_dir, &specifier)
      .map_err(|error| match error {
        nodejs_resolver::Error::Io(error) => Error::Io { source: error },
        nodejs_resolver::Error::UnexpectedJson((json_path, error)) => Error::Anyhow {
          source: anyhow::Error::msg(format!("{:?} in {:?}", error, json_path)),
        },
        nodejs_resolver::Error::UnexpectedValue(error) => Error::Anyhow {
          source: anyhow::Error::msg(error),
        },
        _ => {
          if let Some(importer) = importer {
            let span = args.span.unwrap_or_default();
            let message = if let nodejs_resolver::Error::Overflow = error {
              format!(
                "Can't resolve {:?} in {importer} , maybe it had cycle alias",
                specifier,
              )
            } else {
              format!("Failed to resolve {} in {importer}", specifier)
            };
            Error::TraceableError(TraceableError::from_path(
              importer.to_string(),
              span.start as usize,
              span.end as usize,
              "Resolve error".to_string(),
              message,
            ))
          } else {
            Error::InternalError(format!(
              "Failed to resolve {} in {}",
              specifier,
              context.display()
            ))
          }
        }
      })
  })
  .await
  .map_err(anyhow::Error::from)?
}
