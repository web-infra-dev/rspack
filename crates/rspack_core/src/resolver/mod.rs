mod factory;
mod resolver_impl;

use std::path::PathBuf;

use rspack_error::Error;
use rspack_loader_runner::DescriptionData;

pub use self::factory::{ResolveOptionsWithDependencyType, ResolverFactory};
pub use self::resolver_impl::{ResolveInnerOptions, Resolver};
use crate::{ResolveArgs, SharedPluginDriver};

/// A successful path resolution or an ignored path.
#[derive(Debug, Clone)]
pub enum ResolveResult {
  Resource(Resource),
  Ignored,
}

/// A successful path resolution.
///
/// Contains the raw `package.json` value if there is one.
#[derive(Debug, Clone)]
pub struct Resource {
  pub path: PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
  pub description_data: Option<DescriptionData>,
}

impl Resource {
  /// Get the full path with query and fragment attached.
  pub fn full_path(&self) -> PathBuf {
    let mut buf = format!("{}", self.path.display());
    if let Some(query) = self.query.as_ref() {
      buf.push_str(query);
    }
    if let Some(fragment) = self.fragment.as_ref() {
      buf.push_str(fragment);
    }
    PathBuf::from(buf)
  }
}

/// A runtime error message and an error for rspack stats.
pub struct ResolveError(pub String, pub Error);

/// Main entry point for module resolution.
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
) -> Result<ResolveResult, ResolveError> {
  let mut args = args;

  let resolver = plugin_driver
    .resolver_factory
    .get(ResolveOptionsWithDependencyType {
      resolve_options: args.resolve_options.take(),
      resolve_to_context: args.resolve_to_context,
      dependency_type: args.dependency_type.clone(),
      dependency_category: *args.dependency_category,
    });

  let base_dir = args.context.as_ref();
  let result = resolver.resolve(base_dir, args.specifier);

  let (file_dependencies, missing_dependencies) = resolver.dependencies();
  args.file_dependencies.extend(file_dependencies);
  args.missing_dependencies.extend(missing_dependencies);

  result.map_err(|error| error.into_resolve_error(args, plugin_driver))
}
