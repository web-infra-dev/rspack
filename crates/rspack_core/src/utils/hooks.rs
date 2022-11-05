use crate::{
  NormalModuleFactoryContext, Resolve, ResolveArgs, ResolveKind, ResolveResult, SharedPluginDriver,
};
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
    &plugin_driver.options.context
  };
  tracing::trace!(
    "resolved importer:{:?},specifier:{:?}",
    args.importer,
    args.specifier
  );
  let is_cjs = matches!(
    args.kind,
    ResolveKind::Require | ResolveKind::ModuleHotAccept
  );
  let is_esm = matches!(
    args.kind,
    ResolveKind::Import | ResolveKind::AtImport | ResolveKind::AtImportUrl
  );
  // TODO: should add more test
  let condition_names = if is_esm {
    Some(vec![String::from("..."), String::from("import")])
  } else if is_cjs {
    Some(vec![String::from("..."), String::from("require")])
  } else {
    None
  };
  // TODO: should cache `get`.
  let resolver = plugin_driver.resolver_factory.get(Resolve {
    condition_names,
    ..Default::default()
  });
  let result = resolver.resolve(base_dir, args.specifier);
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
        Error::InternalError(format!("Failed to resolve {}", args.specifier))
      }
    }
  })
}
