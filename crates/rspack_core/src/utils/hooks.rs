use std::path::Path;

use rspack_error::{internal_error, Error, InternalError, Severity, TraceableError};
use sugar_path::SugarPath;

use crate::{ResolveArgs, ResolveOptionsWithDependencyType, ResolveResult, SharedPluginDriver};

/// Tuple used to represent a resolve error.
/// The first element is the error message for runtime and the second element is the error used for stats and so on.
pub struct ResolveError(pub String, pub Error);

pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
  //  _job_context: &mut NormalModuleFactoryContext,
) -> Result<ResolveResult, ResolveError> {
  let plugin_driver = plugin_driver;
  let importer = args.importer.map(|i| i.to_string());
  let base_dir = args.context.as_ref();

  tracing::trace!(
    "resolved importer:{:?},specifier:{:?}",
    args.importer,
    args.specifier
  );

  let resolver = plugin_driver
    .resolver_factory
    .get(ResolveOptionsWithDependencyType {
      resolve_options: args.resolve_options,
      resolve_to_context: args.resolve_to_context,
      dependency_type: args.dependency_type.clone(),
      dependency_category: *args.dependency_category,
    });
  let result = resolver.resolve(base_dir, args.specifier);
  let (file_dependencies, missing_dependencies) = resolver.dependencies();
  args.file_dependencies.extend(file_dependencies);
  args.missing_dependencies.extend(missing_dependencies);

  result.map_err(|error| match error {
    nodejs_resolver::Error::Io(error) => {
      ResolveError(error.to_string(), Error::Io { source: error })
    }
    nodejs_resolver::Error::UnexpectedJson((json_path, error)) => ResolveError(
      format!(
        "{error:?} in {}",
        json_path.relative(&plugin_driver.options.context).display()
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
    nodejs_resolver::Error::CantFindTsConfig(path) => ResolveError(
      format!("{} is not a tsconfig", path.display()),
      internal_error!("{} is not a tsconfig", path.display()),
    ),
    _ => {
      if let Some(importer) = &importer {
        let span = args.span.unwrap_or_default();
        // Use relative path in runtime for stable hashing
        let (runtime_message, internal_message) = if let nodejs_resolver::Error::Overflow = error {
          (
            format!(
              "Can't resolve {:?} in {} , maybe it had cycle alias",
              args.specifier,
              Path::new(&importer)
                .relative(&plugin_driver.options.context)
                .display()
            ),
            format!(
              "Can't resolve {:?} in {} , maybe it had cycle alias",
              args.specifier, importer
            ),
          )
        } else {
          (
            format!(
              "Failed to resolve {} in {}",
              args.specifier,
              base_dir.display()
            ),
            format!("Failed to resolve {} in {}", args.specifier, importer),
          )
        };
        ResolveError(
          runtime_message,
          TraceableError::from_real_file_path(
            Path::new(importer),
            span.start as usize,
            span.end as usize,
            "Resolve error".to_string(),
            internal_message.clone(),
          )
          .map(|e| {
            if args.optional {
              Error::TraceableError(e.with_severity(Severity::Warn))
            } else {
              Error::TraceableError(e)
            }
          })
          .unwrap_or_else(|_| {
            if args.optional {
              Error::InternalError(InternalError::new(internal_message, Severity::Warn))
            } else {
              internal_error!(internal_message)
            }
          }),
        )
      } else {
        ResolveError(
          format!("Failed to resolve {} in project root", args.specifier),
          internal_error!("Failed to resolve {} in project root", args.specifier),
        )
      }
    }
  })
}
