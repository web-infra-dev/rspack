use std::{
  fmt,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{
  error, miette::miette, DiagnosticError, Error, ErrorExt, Severity, TraceableError,
};
use rspack_fs::AsyncReadableFileSystem;
use rspack_loader_runner::DescriptionData;

use super::{ResolveResult, Resource};
use crate::resolver::filesystem::ResolverFileSystem;
use crate::{DependencyCategory, Resolve, ResolveArgs, ResolveOptionsWithDependencyType};

/// Proxy to [nodejs_resolver::Error] or [oxc_resolver::ResolveError]
#[derive(Debug)]
pub enum ResolveInnerError {
  NodejsResolver(nodejs_resolver::Error),
  OxcResolver(oxc_resolver::ResolveError),
}

/// Proxy to [nodejs_resolver::Options] or [oxc_resolver::ResolveOptions]
pub enum ResolveInnerOptions<'a> {
  NodejsResolver(&'a nodejs_resolver::Options),
  OxcResolver(&'a oxc_resolver::ResolveOptions),
}

impl<'a> fmt::Debug for ResolveInnerOptions<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::NodejsResolver(options) => {
        let mut options = (*options).clone();
        options.external_cache = None;
        write!(f, "{:?}", options)
      }
      Self::OxcResolver(options) => {
        write!(f, "{:?}", options)
      }
    }
  }
}

impl<'a> ResolveInnerOptions<'a> {
  pub fn is_enforce_extension_enabled(&self) -> bool {
    match self {
      Self::NodejsResolver(options) => matches!(
        options.enforce_extension,
        nodejs_resolver::EnforceExtension::Enabled
      ),
      Self::OxcResolver(options) => matches!(
        options.enforce_extension,
        oxc_resolver::EnforceExtension::Enabled
      ),
    }
  }

  pub fn extensions(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::NodejsResolver(options) => options.extensions.iter(),
      Self::OxcResolver(options) => options.extensions.iter(),
    }
  }

  pub fn main_files(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::NodejsResolver(options) => options.main_files.iter(),
      Self::OxcResolver(options) => options.main_files.iter(),
    }
  }

  pub fn modules(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::NodejsResolver(options) => options.modules.iter(),
      Self::OxcResolver(options) => options.modules.iter(),
    }
  }
}

/// Proxy to [nodejs_resolver::Resolver] or [oxc_resolver::Resolver]
///
/// Internal caches are shared.
#[derive(Debug)]
pub enum Resolver {
  NodejsResolver(nodejs_resolver::Resolver, Arc<nodejs_resolver::Cache>),
  OxcResolver(oxc_resolver::ResolverGeneric<ResolverFileSystem>),
}

impl Resolver {
  pub fn new(
    new_resolver: bool,
    options: Resolve,
    fs: Option<Arc<dyn AsyncReadableFileSystem + Send + Sync>>,
  ) -> Self {
    if new_resolver {
      Self::new_oxc_resolver(options, fs)
    } else {
      Self::new_nodejs_resolver(options)
    }
  }

  fn new_nodejs_resolver(options: Resolve) -> Self {
    let cache = Arc::new(nodejs_resolver::Cache::default());
    let options =
      to_nodejs_resolver_options(cache.clone(), options, false, DependencyCategory::Unknown);
    let resolver = nodejs_resolver::Resolver::new(options);
    Self::NodejsResolver(resolver, cache)
  }

  fn new_oxc_resolver(
    options: Resolve,
    fs: Option<Arc<dyn AsyncReadableFileSystem + Send + Sync>>,
  ) -> Self {
    let options = to_oxc_resolver_options(options, false, DependencyCategory::Unknown);
    let resolver = match fs {
      Some(fs) => {
        oxc_resolver::ResolverGeneric::new_with_file_system(ResolverFileSystem::new(fs), options)
      }
      None => oxc_resolver::ResolverGeneric::new(options),
    };

    Self::OxcResolver(resolver)
  }

  /// Clear cache for all resolver instances
  pub fn clear_cache(&self) {
    match self {
      Self::NodejsResolver(_, cache) => cache.entries.clear(),
      Self::OxcResolver(resolver) => resolver.clear_cache(),
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
      }
      Self::OxcResolver(resolver) => {
        let options = to_oxc_resolver_options(
          options,
          options_with_dependency_type.resolve_to_context,
          options_with_dependency_type.dependency_category,
        );
        let resolver = resolver.clone_with_options(options);
        Self::OxcResolver(resolver)
      }
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
      Self::OxcResolver(resolver) => ResolveInnerOptions::OxcResolver(resolver.options()),
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
      Self::OxcResolver(resolver) => match resolver.resolve(path, request) {
        Ok(r) => Ok(ResolveResult::Resource(Resource {
          path: r.path().to_path_buf(),
          query: r.query().map(ToString::to_string),
          fragment: r.fragment().map(ToString::to_string),
          description_data: r
            .package_json()
            .map(|d| DescriptionData::new(d.directory().to_path_buf(), Arc::clone(&d.raw_json))),
        })),
        Err(oxc_resolver::ResolveError::Ignored(_)) => Ok(ResolveResult::Ignored),
        Err(error) => Err(ResolveInnerError::OxcResolver(error)),
      },
    }
  }
}

impl ResolveInnerError {
  pub fn into_resolve_error(self, args: &ResolveArgs<'_>) -> Error {
    match self {
      Self::NodejsResolver(error) => map_nodejs_resolver_error(error, args),
      Self::OxcResolver(error) => map_oxc_resolver_error(error, args),
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
  let tsconfig = options.tsconfig.map(|c| c.config_file);
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

fn to_oxc_resolver_options(
  options: Resolve,
  resolve_to_context: bool,
  dependency_type: DependencyCategory,
) -> oxc_resolver::ResolveOptions {
  let options = options.merge_by_dependency(dependency_type);
  let tsconfig = options.tsconfig.map(|c| c.into());
  let enforce_extension = oxc_resolver::EnforceExtension::Auto;
  let description_files = vec!["package.json".to_string()];
  let extensions = options.extensions.unwrap_or_else(|| {
    vec![".tsx", ".ts", ".jsx", ".js", ".mjs", ".json"]
      .into_iter()
      .map(|s| s.to_string())
      .collect()
  });
  let alias = options
    .alias
    .unwrap_or_default()
    .into_iter()
    .map(|(key, value)| {
      let value = value
        .into_iter()
        .map(|x| match x {
          nodejs_resolver::AliasMap::Target(target) => oxc_resolver::AliasValue::Path(target),
          nodejs_resolver::AliasMap::Ignored => oxc_resolver::AliasValue::Ignore,
        })
        .collect();
      (key, value)
    })
    .collect();
  let prefer_relative = options.prefer_relative.unwrap_or(false);
  let symlinks = options.symlinks.unwrap_or(true);
  let main_files = options
    .main_files
    .unwrap_or_else(|| vec![String::from("index")]);
  let main_fields = options
    .main_fields
    .unwrap_or_else(|| vec![String::from("module"), String::from("main")]);
  let alias_fields = (if options.browser_field.unwrap_or(true) {
    vec!["browser".to_string()]
  } else {
    vec![]
  })
  .into_iter()
  .map(|x| vec![x])
  .collect();
  let condition_names = options
    .condition_names
    .unwrap_or_else(|| vec!["module".to_string(), "import".to_string()]);
  let modules = options
    .modules
    .unwrap_or_else(|| vec!["node_modules".to_string()]);
  let fallback = options
    .fallback
    .unwrap_or_default()
    .into_iter()
    .map(|(key, value)| {
      let value = value
        .into_iter()
        .map(|x| match x {
          nodejs_resolver::AliasMap::Target(target) => oxc_resolver::AliasValue::Path(target),
          nodejs_resolver::AliasMap::Ignored => oxc_resolver::AliasValue::Ignore,
        })
        .collect();
      (key, value)
    })
    .collect();
  let fully_specified = options.fully_specified.unwrap_or_default();
  let exports_fields = options
    .exports_field
    .unwrap_or_else(|| vec![vec!["exports".to_string()]]);
  let extension_alias = options.extension_alias.unwrap_or_default();
  oxc_resolver::ResolveOptions {
    fallback,
    modules,
    extensions,
    enforce_extension,
    alias,
    prefer_relative,
    symlinks,
    alias_fields,
    description_files,
    main_files,
    main_fields,
    condition_names,
    tsconfig,
    resolve_to_context,
    fully_specified,
    exports_fields,
    extension_alias,
    // not supported by rspack yet
    prefer_absolute: false,
    restrictions: vec![],
    roots: vec![],
    builtin_modules: false,
  }
}

fn map_nodejs_resolver_error(error: nodejs_resolver::Error, args: &ResolveArgs<'_>) -> Error {
  match error {
    nodejs_resolver::Error::Io(error) => DiagnosticError::from(error.boxed()).into(),
    nodejs_resolver::Error::UnexpectedJson((json_path, error)) => {
      error!("{error:?} in {json_path:?}")
    }
    nodejs_resolver::Error::UnexpectedValue(error) => {
      error!(error)
    }
    nodejs_resolver::Error::CantFindTsConfig(path) => {
      error!("{} is not a tsconfig", path.display())
    }
    _ => {
      let is_recursion = matches!(error, nodejs_resolver::Error::Overflow);
      map_resolver_error(is_recursion, args)
    }
  }
}

fn map_oxc_resolver_error(error: oxc_resolver::ResolveError, args: &ResolveArgs<'_>) -> Error {
  match error {
    oxc_resolver::ResolveError::InvalidPackageTarget(specifier) => {
      let message = format!(
        "Export should be relative path and start with \"./\", but got {}",
        specifier
      );
      error!(message)
    }
    oxc_resolver::ResolveError::IOError(error) => DiagnosticError::from(error.boxed()).into(),
    oxc_resolver::ResolveError::Builtin(error) => {
      error!("Builtin module: {}", error)
    }
    oxc_resolver::ResolveError::Ignored(path) => {
      error!("Path is ignored: {}", path.display())
    }
    oxc_resolver::ResolveError::TsconfigNotFound(path) => {
      error!("{} is not a tsconfig", path.display())
    }
    oxc_resolver::ResolveError::ExtensionAlias => {
      error!("All of the aliased extension are not found")
    }
    oxc_resolver::ResolveError::Specifier(_) => {
      error!("The provided patn specifier cannot be parsed")
    }
    oxc_resolver::ResolveError::JSON(json) => {
      error!("{:?}", json)
    }
    oxc_resolver::ResolveError::Restriction(path) => {
      error!(
        "Restriction by `ResolveOptions::restrictions`: {}",
        path.display()
      )
    }
    oxc_resolver::ResolveError::InvalidModuleSpecifier(error) => {
      error!("Invalid module specifier: {}", error)
    }
    oxc_resolver::ResolveError::PackagePathNotExported(error) => {
      error!("Package subpath '{}' is not defined by \"exports\"", error)
    }
    oxc_resolver::ResolveError::InvalidPackageConfig(path) => {
      error!("Invalid package config in: {}", path.display())
    }
    oxc_resolver::ResolveError::InvalidPackageConfigDefault(path) => {
      error!("Default condition should be last one: {}", path.display())
    }
    oxc_resolver::ResolveError::InvalidPackageConfigDirectory(path) => {
      error!(
        "Expecting folder to folder mapping. \"{}\" should end with \"/\"",
        path.display()
      )
    }
    oxc_resolver::ResolveError::PackageImportNotDefined(error) => {
      error!("Package import not defined: {}", error)
    }
    oxc_resolver::ResolveError::Unimplemented(error) => {
      error!("{} is unimplemented", error)
    }
    oxc_resolver::ResolveError::Recursion => map_resolver_error(true, args),
    oxc_resolver::ResolveError::NotFound(_) => map_resolver_error(false, args),
  }
}

fn map_resolver_error(is_recursion: bool, args: &ResolveArgs<'_>) -> Error {
  let request = &args.specifier;
  let context = &args.context;

  let importer = args.importer;
  if importer.is_none() {
    return miette!("Resolve error: Can't resolve '{request}' in '{context}'");
  }

  let span = args.span.unwrap_or_default();
  let message = format!("Can't resolve '{request}' in '{context}'");
  TraceableError::from_empty_file(
    span.start as usize,
    span.end as usize,
    "Resolve error".to_string(),
    message,
  )
  .with_help(if is_recursion {
    Some("maybe it had cyclic aliases")
  } else {
    None
  })
  .with_severity(
    // See: https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/Compilation.js#L1796
    if args.optional {
      Severity::Warn
    } else {
      Severity::Error
    },
  )
  .into()
}
