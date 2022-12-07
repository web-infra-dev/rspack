use crate::{ErrorSpan, ResolveArgs, ResolveKind, ResolveResult, SharedPluginDriver};
use rspack_error::{Error, Result, TraceableError};
use std::path::{Path, PathBuf};
use tracing::instrument;
use ustr::Ustr;

#[derive(Debug)]
pub struct OwnedResolveArgs {
  pub path: PathBuf,
  pub request: Ustr,
}

#[instrument(name = "resolve", skip_all)]
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
  resolving_tx: tokio::sync::mpsc::UnboundedSender<(
    crate::OwnedResolveArgs,
    tokio::sync::oneshot::Sender<nodejs_resolver::RResult<crate::ResolveResult>>,
  )>,
  //  _job_context: &mut NormalModuleFactoryContext,
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
  let (result_tx, result_rx) =
    tokio::sync::oneshot::channel::<nodejs_resolver::RResult<crate::ResolveResult>>();
  resolving_tx
    .send((
      OwnedResolveArgs {
        path: base_dir.to_path_buf(),
        request: Ustr::from(args.specifier),
      },
      result_tx,
    ))
    .map_err(|e| {
      rspack_error::Error::InternalError(format!("resolving_tx.send failed: {:?}", e))
    })?;

  let result = result_rx.await.map_err(|err| {
    rspack_error::Error::InternalError(format!("result_rx.await failed: {:?}", err))
  })?;

  result.map_err(|error| match error {
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
        Error::InternalError(format!(
          "Failed to resolve {} in {}",
          args.specifier,
          plugin_driver.options.context.display()
        ))
      }
    }
  })
}
