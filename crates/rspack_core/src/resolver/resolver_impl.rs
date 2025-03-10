use std::{
  fmt,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{
  miette::{diagnostic, Diagnostic},
  DiagnosticExt, Severity, TraceableError,
};
use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::DescriptionData;
use rspack_paths::AssertUtf8;
use rspack_util::location::try_line_column_length_to_offset_length;
use rustc_hash::FxHashSet as HashSet;

use super::{boxfs::BoxFS, ResolveResult, Resource};
use crate::{AliasMap, DependencyCategory, Resolve, ResolveArgs, ResolveOptionsWithDependencyType};

#[derive(Debug, Default, Clone)]
pub struct ResolveContext {
  /// Files that was found on file system
  pub file_dependencies: HashSet<PathBuf>,
  /// Dependencies that was not found on file system
  pub missing_dependencies: HashSet<PathBuf>,
}

/// Proxy to [nodejs_resolver::Error] or [rspack_resolver::ResolveError]
#[derive(Debug)]
pub enum ResolveInnerError {
  RspackResolver(rspack_resolver::ResolveError),
}

/// Proxy to [rspack_resolver::ResolveOptions]
pub enum ResolveInnerOptions<'a> {
  RspackResolver(&'a rspack_resolver::ResolveOptions),
}

impl fmt::Debug for ResolveInnerOptions<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::RspackResolver(options) => {
        write!(f, "{options:?}")
      }
    }
  }
}

impl ResolveInnerOptions<'_> {
  pub fn is_enforce_extension_enabled(&self) -> bool {
    match self {
      Self::RspackResolver(options) => matches!(
        options.enforce_extension,
        rspack_resolver::EnforceExtension::Enabled
      ),
    }
  }

  pub fn extensions(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::RspackResolver(options) => options.extensions.iter(),
    }
  }

  pub fn main_files(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::RspackResolver(options) => options.main_files.iter(),
    }
  }

  pub fn modules(&self) -> impl Iterator<Item = &String> {
    match self {
      Self::RspackResolver(options) => options.modules.iter(),
    }
  }
}

/// Proxy to [rspack_resolver::Resolver]
///
/// Internal caches are shared.
#[derive(Debug)]
pub struct Resolver {
  resolver: rspack_resolver::ResolverGeneric<BoxFS>,
}

impl Resolver {
  pub fn new(options: Resolve, fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self::new_rspack_resolver(options, fs)
  }

  fn new_rspack_resolver(options: Resolve, fs: Arc<dyn ReadableFileSystem>) -> Self {
    let options = to_rspack_resolver_options(options, false, DependencyCategory::Unknown);
    let boxfs = BoxFS::new(fs);
    let resolver = rspack_resolver::ResolverGeneric::new_with_file_system(boxfs, options);
    Self { resolver }
  }

  /// Clear cache for all resolver instances
  pub fn clear_cache(&self) {
    self.resolver.clear_cache();
  }

  /// Create a new resolver by cloning its internal cache.
  pub fn clone_with_options(
    &self,
    options: Resolve,
    options_with_dependency_type: &ResolveOptionsWithDependencyType,
  ) -> Self {
    let resolver = &self.resolver;
    let options = to_rspack_resolver_options(
      options,
      options_with_dependency_type.resolve_to_context,
      options_with_dependency_type.dependency_category,
    );

    let resolver = resolver.clone_with_options(options);
    Self { resolver }
  }

  /// Return the options from the resolver
  pub fn options(&self) -> ResolveInnerOptions<'_> {
    ResolveInnerOptions::RspackResolver(self.resolver.options())
  }

  /// Resolve a specifier to a given path.
  pub fn resolve(&self, path: &Path, request: &str) -> Result<ResolveResult, ResolveInnerError> {
    match self.resolver.resolve(path, request) {
      Ok(r) => Ok(ResolveResult::Resource(Resource {
        path: r.path().to_path_buf().assert_utf8(),
        query: r.query().unwrap_or_default().to_string(),
        fragment: r.fragment().unwrap_or_default().to_string(),
        description_data: r
          .package_json()
          .map(|d| DescriptionData::new(d.directory().to_path_buf(), Arc::clone(d.raw_json()))),
      })),
      Err(rspack_resolver::ResolveError::Ignored(_)) => Ok(ResolveResult::Ignored),
      Err(error) => Err(ResolveInnerError::RspackResolver(error)),
    }
  }

  /// Resolve a specifier to a given path.
  pub fn resolve_with_context(
    &self,
    path: &Path,
    request: &str,
    resolve_context: &mut ResolveContext,
  ) -> Result<ResolveResult, ResolveInnerError> {
    let resolver = &self.resolver;
    let mut context = Default::default();
    let result = resolver.resolve_with_context(path, request, &mut context);
    resolve_context
      .file_dependencies
      .extend(context.file_dependencies);
    resolve_context
      .missing_dependencies
      .extend(context.missing_dependencies);
    match result {
      Ok(r) => Ok(ResolveResult::Resource(Resource {
        path: r.path().to_path_buf().assert_utf8(),
        query: r.query().unwrap_or_default().to_string(),
        fragment: r.fragment().unwrap_or_default().to_string(),
        description_data: r
          .package_json()
          .map(|d| DescriptionData::new(d.directory().to_path_buf(), Arc::clone(d.raw_json()))),
      })),
      Err(rspack_resolver::ResolveError::Ignored(_)) => Ok(ResolveResult::Ignored),
      Err(error) => Err(ResolveInnerError::RspackResolver(error)),
    }
  }
}

impl ResolveInnerError {
  pub fn into_resolve_error(self, args: &ResolveArgs<'_>) -> Box<dyn Diagnostic + Send + Sync> {
    match self {
      Self::RspackResolver(error) => map_rspack_resolver_error(error, args),
    }
  }
}

fn to_rspack_resolver_options(
  options: Resolve,
  resolve_to_context: bool,
  dependency_type: DependencyCategory,
) -> rspack_resolver::ResolveOptions {
  let options = options.merge_by_dependency(dependency_type);
  let tsconfig = options.tsconfig.map(|c| c.into());
  let enforce_extension =
    options
      .enforce_extension
      .map_or(rspack_resolver::EnforceExtension::Auto, |e| match e {
        true => rspack_resolver::EnforceExtension::Enabled,
        false => rspack_resolver::EnforceExtension::Disabled,
      });
  let description_files = options
    .description_files
    .unwrap_or_else(|| vec!["package.json".to_string()]);
  let imports_fields = options
    .imports_fields
    .unwrap_or_else(|| vec![vec!["imports".to_string()]]);
  let extensions = options
    .extensions
    .unwrap_or_else(|| vec![".js".to_string(), ".json".to_string(), ".wasm".to_string()]);
  let alias = options
    .alias
    .unwrap_or_default()
    .into_iter()
    .map(|(key, value)| {
      let value = value
        .into_iter()
        .map(|x| match x {
          AliasMap::Path(target) => rspack_resolver::AliasValue::Path(target),
          AliasMap::Ignore => rspack_resolver::AliasValue::Ignore,
        })
        .collect();
      (key, value)
    })
    .collect();
  let prefer_relative = options.prefer_relative.unwrap_or(false);
  let prefer_absolute = options.prefer_absolute.unwrap_or(false);
  let symlinks = options.symlinks.unwrap_or(true);
  let main_files = options
    .main_files
    .unwrap_or_else(|| vec![String::from("index")]);
  let main_fields = options
    .main_fields
    .unwrap_or_else(|| vec![String::from("module"), String::from("main")]);
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
          AliasMap::Path(target) => rspack_resolver::AliasValue::Path(target),
          AliasMap::Ignore => rspack_resolver::AliasValue::Ignore,
        })
        .collect();
      (key, value)
    })
    .collect();
  let fully_specified = options.fully_specified.unwrap_or_default();
  let exports_fields = options
    .exports_fields
    .unwrap_or_else(|| vec![vec!["exports".to_string()]]);
  let extension_alias = options.extension_alias.unwrap_or_default();
  let alias_fields = options
    .alias_fields
    .unwrap_or_else(|| vec![vec![String::from("browser")]]);
  let restrictions = options
    .restrictions
    .unwrap_or_default()
    .into_iter()
    .map(|s| rspack_resolver::Restriction::Path(s.into()))
    .collect();
  let roots = options
    .roots
    .unwrap_or_default()
    .into_iter()
    .map(PathBuf::from)
    .collect();

  rspack_resolver::ResolveOptions {
    fallback,
    modules,
    extensions,
    enforce_extension,
    alias,
    prefer_relative,
    prefer_absolute,
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
    restrictions,
    roots,
    builtin_modules: false,
    imports_fields,
    enable_pnp: options.pnp.unwrap_or(false),
  }
}

fn map_rspack_resolver_error(
  error: rspack_resolver::ResolveError,
  args: &ResolveArgs<'_>,
) -> Box<dyn Diagnostic + Send + Sync> {
  match error {
    rspack_resolver::ResolveError::IOError(error) => diagnostic!("{}", error).boxed(),
    rspack_resolver::ResolveError::Recursion => map_resolver_error(true, args),
    rspack_resolver::ResolveError::NotFound(_) => map_resolver_error(false, args),
    rspack_resolver::ResolveError::JSON(error) => {
      if let Some(content) = &error.content {
        let rope = ropey::Rope::from(&**content);
        let Some((offset, _)) =
          try_line_column_length_to_offset_length(&rope, error.line, error.column, 0)
        else {
          return diagnostic!(
            "JSON parse error: {:?} in '{}'",
            error,
            error.path.display()
          )
          .boxed();
        };
        drop(rope);

        fn ceil_char_boundary(content: &str, mut index: usize) -> usize {
          if index > content.len() {
            return content.len();
          }

          while !content.is_char_boundary(index) {
            if index == 0 {
              return 0;
            }
            index = index.saturating_sub(1);
          }

          index
        }

        let offset = ceil_char_boundary(content, offset);

        if content[offset..].starts_with('\u{feff}') {
          return TraceableError::from_file(
            content.clone(),
            offset,
            offset,
            "JSON parse error".to_string(),
            format!("BOM character found in '{}'", error.path.display()),
          )
          .boxed();
        }

        TraceableError::from_file(
          content.clone(),
          offset,
          offset,
          "JSON parse error".to_string(),
          format!("{} in '{}'", error.message, error.path.display()),
        )
        .boxed()
      } else {
        diagnostic!(
          "JSON parse error: {:?} in '{}'",
          error,
          error.path.display()
        )
        .boxed()
      }
    }
    _ => diagnostic!("{}", error).boxed(),
  }
}

fn map_resolver_error(
  is_recursion: bool,
  args: &ResolveArgs<'_>,
) -> Box<dyn Diagnostic + Send + Sync> {
  let request = &args.specifier;
  let context = &args.context;

  let importer = args.importer;
  if importer.is_none() {
    return diagnostic!("Module not found: Can't resolve '{request}' in '{context}'").boxed();
  }

  let span = args.span.unwrap_or_default();
  let message = format!("Can't resolve '{request}' in '{context}'");
  TraceableError::from_lazy_file(
    span.start as usize,
    span.end as usize,
    "Module not found".to_string(),
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
  .boxed()
}
