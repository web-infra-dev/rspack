use std::path::Path;

use rspack_error::{internal_error, Error, TraceableError};
use sugar_path::SugarPath;
use tracing::instrument;

use crate::{DependencyType, Resolve, ResolveArgs, ResolveResult, SharedPluginDriver};

/// Tuple used to represent a resolve error.
/// The first element is the error message for runtime and the second element is the error used for stats and so on.
pub struct ResolveError(pub String, pub Error);

#[instrument(name = "resolve", skip_all)]
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
  //  _job_context: &mut NormalModuleFactoryContext,
) -> Result<ResolveResult, ResolveError> {
  let plugin_driver = plugin_driver.read().await;
  let importer = args.importer.map(|i| i.display().to_string());
  let base_dir = if let Some(i) = importer.as_ref() {
    {
      // TODO: delete this fn after use `normalModule.context` rather than `importer`
      if let Some(index) = i.find('?') {
        Path::new(&i[0..index])
      } else {
        Path::new(i)
      }
    }
    .parent()
    .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))
    .map_err(|err| ResolveError(format!("parent() failed for {importer:?}"), err.into()))?
  } else {
    &plugin_driver.options.context
  };

  tracing::trace!(
    "resolved importer:{:?},specifier:{:?}",
    args.importer,
    args.specifier
  );

  let result = if let Some(options) = args.resolve_options {
    let resolver = plugin_driver.resolver_factory.get(options);
    let res = resolver.resolve(base_dir, args.specifier);

    let (file_dependencies, missing_dependencies) = resolver.dependencies();
    args.file_dependencies.extend(file_dependencies);
    args.missing_dependencies.extend(missing_dependencies);

    res
  } else if plugin_driver.options.resolve.condition_names.is_none() {
    let is_esm = matches!(
      args.dependency_type,
      DependencyType::EsmImport
        | DependencyType::DynamicImport
        | DependencyType::ImportMetaHotAccept
    );
    let condition_names = if is_esm {
      vec![
        String::from("import"),
        String::from("module"),
        String::from("webpack"),
        String::from("development"),
        String::from("browser"),
      ]
    } else {
      vec![
        String::from("require"),
        String::from("module"),
        String::from("webpack"),
        String::from("development"),
        String::from("browser"),
      ]
    };
    let options = Resolve {
      condition_names: Some(condition_names),
      ..plugin_driver.options.resolve.clone()
    };
    let resolver = plugin_driver.resolver_factory.get(options);
    let res = resolver.resolve(base_dir, args.specifier);

    let (file_dependencies, missing_dependencies) = resolver.dependencies();
    args.file_dependencies.extend(file_dependencies);
    args.missing_dependencies.extend(missing_dependencies);

    res
  } else {
    let res = plugin_driver.resolver.resolve(base_dir, args.specifier);

    let (file_dependencies, missing_dependencies) = plugin_driver.resolver.dependencies();
    args.file_dependencies.extend(file_dependencies);
    args.missing_dependencies.extend(missing_dependencies);

    res
  };

  result.map_err(|error| match error {
    nodejs_resolver::Error::Io(error) => {
      ResolveError(error.to_string(), Error::Io { source: error })
    }
    nodejs_resolver::Error::UnexpectedJson((json_path, error)) => ResolveError(
      format!(
        "{error:?} in {}",
        json_path.relative(&args.compiler_options.context).display()
      ),
      Error::Anyhow {
        source: anyhow::Error::msg(format!("{error:?} in {json_path:?}")),
      },
    ),
    nodejs_resolver::Error::UnexpectedValue(error) => ResolveError(
      error.clone(),
      Error::Anyhow {
        source: anyhow::Error::msg(error),
      },
    ),
    _ => {
      if let Some(importer) = args.importer {
        let span = args.span.unwrap_or_default();

        // Use relative path in runtime for stable hashing
        let (runtime_message, internal_message) = if let nodejs_resolver::Error::Overflow = error {
          (
            format!(
              "Can't resolve {:?} in {} , maybe it had cycle alias",
              args.specifier,
              importer.relative(&args.compiler_options.context).display()
            ),
            format!(
              "Can't resolve {:?} in {} , maybe it had cycle alias",
              args.specifier,
              importer.display()
            ),
          )
        } else {
          (
            format!(
              "Failed to resolve {} in {}",
              args.specifier,
              importer.relative(&args.compiler_options.context).display()
            ),
            format!(
              "Failed to resolve {} in {}",
              args.specifier,
              importer.display()
            ),
          )
        };
        ResolveError(
          runtime_message,
          Error::TraceableError(
            TraceableError::from_path(
              importer.display().to_string(),
              span.start as usize,
              span.end as usize,
              "Resolve error".to_string(),
              internal_message,
            )
            .into(),
          ),
        )
      } else {
        ResolveError(
          "Failed to resolve {} in context".to_owned(),
          Error::InternalError(internal_error!(format!(
            "Failed to resolve {} in {}",
            args.specifier,
            args.compiler_options.context.display()
          ))),
        )
      }
    }
  })
}
