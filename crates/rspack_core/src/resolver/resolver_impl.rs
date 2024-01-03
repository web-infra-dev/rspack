use std::{
  fmt,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{
  internal_error, DiagnosticError, Error, ErrorExt, InternalError, Severity, TraceableError,
};
use rspack_loader_runner::DescriptionData;

use super::{ResolveResult, Resource};
use crate::{AliasMap, DependencyCategory, Resolve, ResolveArgs, ResolveOptionsWithDependencyType};

/// Proxy to [oxc_resolver::ResolveError]
#[derive(Debug)]
pub enum ResolveInnerError {
  OxcResolver(oxc_resolver::ResolveError),
}

/// Proxy to [oxc_resolver::ResolveOptions]
pub enum ResolveInnerOptions<'a> {
  OxcResolver(&'a oxc_resolver::ResolveOptions),
}

impl<'a> fmt::Debug for ResolveInnerOptions<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::OxcResolver(options) => {
        write!(f, "{:?}", options)
      }
    }
  }
}

impl<'a> ResolveInnerOptions<'a> {
  pub fn is_enforce_extension_enabled(&self) -> bool {
    match self {
      Self::OxcResolver(options) => matches!(
        options.enforce_extension,
        oxc_resolver::EnforceExtension::Enabled
      ),
    }
  }

  pub fn extensions(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::OxcResolver(options) => options.extensions.iter(),
    }
  }

  pub fn main_files(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::OxcResolver(options) => options.main_files.iter(),
    }
  }

  pub fn modules(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::OxcResolver(options) => options.modules.iter(),
    }
  }
}

/// Proxy to [oxc_resolver::Resolver]
///
/// Internal caches are shared.
#[derive(Debug)]
pub enum Resolver {
  OxcResolver(oxc_resolver::Resolver),
}

impl Resolver {
  pub fn new(options: Resolve) -> Self {
    Self::new_oxc_resolver(options)
  }

  fn new_oxc_resolver(options: Resolve) -> Self {
    let options = to_oxc_resolver_options(options, false, DependencyCategory::Unknown);
    let resolver = oxc_resolver::Resolver::new(options);
    Self::OxcResolver(resolver)
  }

  /// Clear cache for all resolver instances
  pub fn clear_cache(&self) {
    match self {
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
      Self::OxcResolver(resolver) => ResolveInnerOptions::OxcResolver(resolver.options()),
    }
  }

  /// Resolve a specifier to a given path.
  pub fn resolve(&self, path: &Path, request: &str) -> Result<ResolveResult, ResolveInnerError> {
    match self {
      Self::OxcResolver(resolver) => match resolver.resolve(path, request) {
        Ok(r) => Ok(ResolveResult::Resource(Resource {
          path: r.path().to_path_buf(),
          query: r.query().map(ToString::to_string),
          fragment: r.fragment().map(ToString::to_string),
          description_data: r
            .package_json()
            .map(|d| DescriptionData::new(d.directory().to_path_buf(), Arc::clone(d.raw_json()))),
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
      Self::OxcResolver(error) => map_oxc_resolver_error(error, args),
    }
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
          AliasMap::Path(target) => oxc_resolver::AliasValue::Path(target),
          AliasMap::Ignore => oxc_resolver::AliasValue::Ignore,
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
          AliasMap::Path(target) => oxc_resolver::AliasValue::Path(target),
          AliasMap::Ignore => oxc_resolver::AliasValue::Ignore,
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

fn map_oxc_resolver_error(error: oxc_resolver::ResolveError, args: &ResolveArgs<'_>) -> Error {
  match error {
    oxc_resolver::ResolveError::IOError(error) => DiagnosticError::from(error.boxed()).into(),
    oxc_resolver::ResolveError::Recursion => map_resolver_error(true, args),
    oxc_resolver::ResolveError::NotFound(_) => map_resolver_error(false, args),
    _ => internal_error!("{}", error),
  }
}

fn map_resolver_error(is_recursion: bool, args: &ResolveArgs<'_>) -> Error {
  let importer = args.importer.map(|i| i.as_str());
  if let Some(importer) = &importer {
    let span = args.span.unwrap_or_default();
    // Use relative path in runtime for stable hashing
    let internal_message = if is_recursion {
      format!(
        "Can't resolve {:?} in {} , maybe it had cycle alias",
        args.specifier, importer
      )
    } else {
      format!("Failed to resolve {} in {}", args.specifier, importer)
    };
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
        e.with_severity(Severity::Warn).into()
      } else {
        e.into()
      }
    })
    .unwrap_or_else(|_| {
      if args.optional {
        InternalError::new(internal_message, Severity::Warn).into()
      } else {
        internal_error!(internal_message)
      }
    })
  } else {
    internal_error!("Failed to resolve {} in project root", args.specifier)
  }
}
