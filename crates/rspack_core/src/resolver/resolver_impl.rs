use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{internal_error, Error, InternalError, Severity, TraceableError};
use rspack_loader_runner::DescriptionData;
use sugar_path::SugarPath;

use super::{ResolveError, ResolveResult, Resource};
use crate::{
  DependencyCategory, Resolve, ResolveArgs, ResolveOptionsWithDependencyType, SharedPluginDriver,
};

/// Proxy to [nodejs_resolver::Error] or [oxc_resolver::ResolveError]
#[derive(Debug)]
pub enum ResolveInnerError {
  NodejsResolver(nodejs_resolver::Error),
  // OxcResolver(oxc_resolver::ResolveError),
}

/// Proxy to [nodejs_resolver::Options] or [oxc_resolver::ResolveOptions]
#[derive(Debug)]
pub enum ResolveInnerOptions<'a> {
  NodejsResolver(&'a nodejs_resolver::Options),
  // OxcResolver(&'a oxc_resolver::ResolveOptions),
}

impl<'a> ResolveInnerOptions<'a> {
  pub fn is_enforce_extension_enabled(&self) -> bool {
    match self {
      Self::NodejsResolver(options) => matches!(
        options.enforce_extension,
        nodejs_resolver::EnforceExtension::Enabled
      ),
      // Self::OxcResolver(options) => matches!(
      // options.enforce_extension,
      // oxc_resolver::EnforceExtension::Enabled
      // ),
    }
  }

  pub fn extensions(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::NodejsResolver(options) => options.extensions.iter(),
      // Self::OxcResolver(options) => options.extensions.iter(),
    }
  }

  pub fn main_files(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::NodejsResolver(options) => options.main_files.iter(),
      // Self::OxcResolver(options) => options.main_files.iter(),
    }
  }

  pub fn modules(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::NodejsResolver(options) => options.modules.iter(),
      // Self::OxcResolver(options) => options.modules.iter(),
    }
  }
}

/// Proxy to [nodejs_resolver::Resolver] or [oxc_resolver::Resolver]
///
/// Internal caches are shared.
#[derive(Debug)]
pub enum Resolver {
  NodejsResolver(nodejs_resolver::Resolver, Arc<nodejs_resolver::Cache>),
  // OxcResolver(oxc_resolver::Resolver),
}

impl Resolver {
  pub fn new(options: Resolve) -> Self {
    Self::new_nodejs_resolver(options)
  }

  fn new_nodejs_resolver(options: Resolve) -> Self {
    let cache = Arc::new(nodejs_resolver::Cache::default());
    let options =
      to_nodejs_resolver_options(cache.clone(), options, false, DependencyCategory::Unknown);
    let resolver = nodejs_resolver::Resolver::new(options);
    Self::NodejsResolver(resolver, cache)
  }

  /// Clear cache for all resolver instances
  pub fn clear_cache(&self) {
    match self {
      Self::NodejsResolver(_, cache) => cache.entries.clear(),
      // Self::OxcResolver(resolver) => resolver.clear_cache(),
    }
  }

  /// Create a new resolver by cloning its internal cache.
  pub fn clone_with_options(
    &self,
    options: Resolve,
    options_with_dependency_type: &ResolveOptionsWithDependencyType,
  ) -> Self {
    match self {
      Self::NodejsResolver(_, cache) => {
        let options = to_nodejs_resolver_options(
          cache.clone(),
          options,
          options_with_dependency_type.resolve_to_context,
          options_with_dependency_type.dependency_category,
        );
        let resolver = nodejs_resolver::Resolver::new(options);
        Self::NodejsResolver(resolver, cache.clone())
      } /* Self::OxcResolver(_resolver) => {
         * unimplemented!()
         * } */
    }
  }

  /// Return `dependencies` from `enhanced-resolve`
  ///
  /// Implementation is currently blank.
  pub fn dependencies(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
    (vec![], vec![])
  }

  /// Return the options from the resolver
  pub fn options(&self) -> ResolveInnerOptions<'_> {
    match self {
      Self::NodejsResolver(resolver, _) => ResolveInnerOptions::NodejsResolver(&resolver.options),
      // Self::OxcResolver(resolver) => ResolveInnerOptions::OxcResolver(resolver.options()),
    }
  }

  /// Resolve a specifier to a given path.
  pub fn resolve(&self, path: &Path, request: &str) -> Result<ResolveResult, ResolveInnerError> {
    match self {
      Self::NodejsResolver(resolver, _) => resolver
        .resolve(path, request)
        .map(|result| match result {
          nodejs_resolver::ResolveResult::Resource(r) => ResolveResult::Resource(Resource {
            path: r.path,
            query: r.query,
            fragment: r.fragment,
            description_data: r.description.map(|d| {
              DescriptionData::new(d.dir().as_ref().to_path_buf(), Arc::clone(d.data().raw()))
            }),
          }),
          nodejs_resolver::ResolveResult::Ignored => ResolveResult::Ignored,
        })
        .map_err(ResolveInnerError::NodejsResolver),
      // Self::OxcResolver(_resolver) => {
      // unimplemented!()
      // }
    }
  }
}

impl ResolveInnerError {
  pub fn into_resolve_error(
    self,
    args: ResolveArgs<'_>,
    plugin_driver: &SharedPluginDriver,
  ) -> ResolveError {
    match self {
      Self::NodejsResolver(error) => map_nodejs_resolver_error(error, args, plugin_driver),
      // Self::OxcResolver(_error) => {
      // unimplemented!()
      // }
    }
  }
}

fn to_nodejs_resolver_options(
  cache: Arc<nodejs_resolver::Cache>,
  options: Resolve,
  resolve_to_context: bool,
  dependency_type: DependencyCategory,
) -> nodejs_resolver::Options {
  let options = options.merge_by_dependency(dependency_type);
  let tsconfig = options.tsconfig;
  let enforce_extension = nodejs_resolver::EnforceExtension::Auto;
  let external_cache = Some(cache);
  let description_file = String::from("package.json");
  let extensions = options.extensions.unwrap_or_else(|| {
    vec![".tsx", ".ts", ".jsx", ".js", ".mjs", ".json"]
      .into_iter()
      .map(|s| s.to_string())
      .collect()
  });
  let alias = options.alias.unwrap_or_default();
  let prefer_relative = options.prefer_relative.unwrap_or(false);
  let symlinks = options.symlinks.unwrap_or(true);
  let main_files = options
    .main_files
    .unwrap_or_else(|| vec![String::from("index")]);
  let main_fields = options
    .main_fields
    .unwrap_or_else(|| vec![String::from("module"), String::from("main")]);
  let browser_field = options.browser_field.unwrap_or(true);
  let condition_names = std::collections::HashSet::from_iter(
    options
      .condition_names
      .unwrap_or_else(|| vec!["module".to_string(), "import".to_string()]),
  );
  let modules = options
    .modules
    .unwrap_or_else(|| vec!["node_modules".to_string()]);
  let fallback = options.fallback.unwrap_or_default();
  let fully_specified = options.fully_specified.unwrap_or_default();
  let exports_field = options
    .exports_field
    .unwrap_or_else(|| vec![vec!["exports".to_string()]]);
  let extension_alias = options.extension_alias.unwrap_or_default();
  nodejs_resolver::Options {
    fallback,
    modules,
    extensions,
    enforce_extension,
    alias,
    prefer_relative,
    external_cache,
    symlinks,
    description_file,
    main_files,
    main_fields,
    browser_field,
    condition_names,
    tsconfig,
    resolve_to_context,
    fully_specified,
    exports_field,
    extension_alias,
  }
}

fn map_nodejs_resolver_error(
  error: nodejs_resolver::Error,
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
) -> ResolveError {
  let base_dir: &Path = args.context.as_ref();
  let importer = args.importer.map(|i| i.as_str());
  match error {
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
            Path::new(
              importer
                .split_once('|')
                .map(|(_, path)| path)
                .unwrap_or(importer),
            ),
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
  }
}
