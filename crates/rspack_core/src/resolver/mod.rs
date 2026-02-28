mod boxfs;
mod factory;
mod resolver_impl;
use std::{
  borrow::Borrow,
  fmt,
  path::PathBuf,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_error::Error;
use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::{DescriptionData, ResourceData};
use rspack_paths::{AssertUtf8, Utf8PathBuf};
use rspack_util::identifier::insert_zero_width_space_for_fragment;
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;

pub use self::{
  factory::{ResolveOptionsWithDependencyType, ResolverFactory},
  resolver_impl::{ResolveContext, ResolveInnerError, ResolveInnerOptions, Resolver},
};
use crate::{
  Context, DependencyCategory, DependencyRange, DependencyType, ModuleIdentifier, Resolve,
  SharedPluginDriver,
};

static RELATIVE_PATH_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^\.\.?\/").expect("should init regex"));

static CURRENT_DIR_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\.[\/])").expect("should init regex"));

#[derive(Debug)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a ModuleIdentifier>,
  pub issuer: Option<&'a str>,
  pub context: Context,
  pub specifier: &'a str,
  pub dependency_type: &'a DependencyType,
  pub dependency_category: &'a DependencyCategory,
  pub span: Option<DependencyRange>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub resolve_to_context: bool,
  pub optional: bool,
  pub file_dependencies: &'a mut FxHashSet<PathBuf>,
  pub missing_dependencies: &'a mut FxHashSet<PathBuf>,
}

/// A successful path resolution or an ignored path.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResolveResult {
  Resource(Resource),
  Ignored,
}

/// A successful path resolution.
///
/// Contains the raw `package.json` value if there is one.
#[derive(Clone)]
pub struct Resource {
  pub path: Utf8PathBuf,
  pub query: String,
  pub fragment: String,
  pub description_data: Option<DescriptionData>,
}

impl fmt::Debug for Resource {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.full_path())
  }
}

impl PartialEq for Resource {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path && self.query == other.query && self.fragment == other.fragment
  }
}
impl Eq for Resource {}

impl Resource {
  /// Get the full path with query and fragment attached.
  pub fn full_path(&self) -> String {
    let mut buf = insert_zero_width_space_for_fragment(self.path.as_str()).into_owned();
    buf.push_str(&insert_zero_width_space_for_fragment(&self.query));
    buf.push_str(&self.fragment);
    buf
  }
}

impl From<Resource> for ResourceData {
  fn from(resource: Resource) -> Self {
    let mut resource_data = Self::new_with_path(
      resource.full_path(),
      resource.path,
      Some(resource.query),
      Some(resource.fragment),
    );
    resource_data.set_description_optional(resource.description_data);
    resource_data
  }
}

pub async fn resolve_for_error_hints(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
  fs: Arc<dyn ReadableFileSystem>,
) -> Option<String> {
  let dep = ResolveOptionsWithDependencyType {
    resolve_options: args
      .resolve_options
      .map(|r| Box::new(Arc::unwrap_or_clone(r))),
    resolve_to_context: args.resolve_to_context,
    dependency_category: *args.dependency_category,
  };

  let base_dir = args.context.clone();
  let base_dir = base_dir.as_ref();

  let fully_specified = dep
    .resolve_options
    .as_ref()
    .and_then(|o| o.fully_specified(Some(args.dependency_category)))
    .unwrap_or_default();

  let prefer_relative = dep
    .resolve_options
    .as_ref()
    .and_then(|o| o.prefer_relative(Some(args.dependency_category)))
    .unwrap_or_default();

  // Try to resolve without fully specified
  if fully_specified {
    let mut dep = dep.clone();
    dep.resolve_options = dep.resolve_options.map(|mut options| {
      options.fully_specified = Some(false);
      options
    });
    let resolver = plugin_driver.resolver_factory.get(dep);
    if let Ok(ResolveResult::Resource(resource)) = resolver.resolve(base_dir, args.specifier).await
    {
      let relative_path = resource
        .path
        .as_std_path()
        .relative(args.context)
        .assert_utf8();
      let suggestion = if let Some((_, [prefix])) = CURRENT_DIR_REGEX
        .captures_iter(args.specifier)
        .next()
        .map(|c| c.extract())
      {
        // If the specifier is a relative path pointing to the current directory,
        // we can suggest the path relative to the current directory.
        format!("{prefix}{relative_path}")
      } else if args.specifier.starts_with("../") || args.specifier.starts_with("..\\") {
        // If the specifier is a relative path to which the parent directory is,
        // then we return the relative path directly.
        relative_path.as_str().to_string()
      } else {
        // If the specifier is a package name like or some arbitrary alias,
        // then we return the full path.
        resource.path.as_str().to_string()
      };
      return Some(format!("Did you mean '{}'?

The request '{}' failed to resolve only because it was resolved as fully specified,
probably because the origin is strict EcmaScript Module,
e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\"type\": \"module\"'.

The extension in the request is mandatory for it to be fully specified.
Add the extension to the request.", suggestion, args.specifier));
    }
  }

  // Try to resolve with relative path if request is not relative
  if !RELATIVE_PATH_REGEX.is_match(args.specifier) && !prefer_relative {
    let dep = dep.clone();
    let module_directories = dep
      .resolve_options
      .as_deref()
      .or(Some(&plugin_driver.options.resolve))
      .and_then(|o| o.modules.as_ref().map(|m| m.join(", ")));
    let module_directories = {
      if let Some(module_directories) = module_directories {
        format!(" ({module_directories}).")
      } else {
        ".".to_string()
      }
    };
    let resolver = plugin_driver.resolver_factory.get(dep);
    let request = format!("./{}", args.specifier);
    if resolver.resolve(base_dir, &request).await.is_ok() {
      return Some(format!(
          "Did you mean './{}'?

Requests that should resolve in the current directory need to start with './'.
Requests that start with a name are treated as module requests and resolve within module directories{module_directories}

If changing the source code is not an option, there is also a resolve options called 'preferRelative'
which tries to resolve these kind of requests in the current directory too.",
          args.specifier
        ));
    }
  }

  // try to resolve relative path with extension
  if RELATIVE_PATH_REGEX.is_match(args.specifier) {
    let connected_path = base_dir.join(args.specifier);
    let normalized_path = connected_path.absolutize();

    let mut is_resolving_dir = false; // whether the request is to resolve a directory or not

    let file_name = normalized_path.file_name();
    let utf8_normalized_path = Utf8PathBuf::from_path_buf(normalized_path.to_path_buf())
      .expect("should be a valid utf8 path");

    let parent_path = match fs.metadata(&utf8_normalized_path).await {
      Ok(metadata) => {
        // if the path is not directory, we need to resolve the parent directory
        if !metadata.is_directory {
          normalized_path.parent()
        } else {
          is_resolving_dir = true;
          Some(normalized_path.borrow())
        }
      }
      Err(_) => normalized_path.parent(),
    };

    if let Some(file_name) = file_name
      && let Some(parent_path) =
        parent_path.and_then(|path| Utf8PathBuf::from_path_buf(path.to_path_buf()).ok())
    {
      // read the files in the parent directory
      if let Ok(files) = fs.read_dir(&parent_path).await {
        let mut requested_names = vec![
          file_name
            .to_str()
            .map(|f| f.to_string())
            .unwrap_or_default(),
        ];
        if is_resolving_dir {
          // The request maybe is like `./` or `./dir` to resolve the main file (e.g.: index) in directory
          // So we need to check them.
          let main_files = dep
            .resolve_options
            .as_deref()
            .or(Some(&plugin_driver.options.resolve))
            .and_then(|o| o.main_files.as_ref().cloned())
            .unwrap_or_default();

          requested_names.extend(main_files);
        }

        let suggestions = files
          .into_iter()
          .filter_map(|file| {
            let path = parent_path.join(file);
            path.file_stem().and_then(|file_stem| {
              if requested_names.contains(&file_stem.to_string()) {
                let mut suggestion = path.as_std_path().relative(&args.context).assert_utf8();

                if !suggestion.as_str().starts_with('.') {
                  suggestion = Utf8PathBuf::from(format!("./{suggestion}"));
                }
                Some(suggestion)
              } else {
                None
              }
            })
          })
          .collect::<Vec<_>>();

        if suggestions.is_empty() {
          return None;
        }

        let mut hint: Vec<String> = vec![];
        for suggestion in suggestions {
          let suggestion_ext = suggestion.extension().unwrap_or_default();
          let specifier = args.specifier;

          hint.push(format!(
          "Found module '{suggestion}'. However, it's not possible to request this module without the extension 
if its extension was not listed in the `resolve.extensions`. Here're some possible solutions:

1. add the extension `\".{suggestion_ext}\"` to `resolve.extensions` in your rspack configuration
2. use '{suggestion}' instead of '{specifier}'
"));
        }

        return Some(hint.join("\n"));
      }
    }
  }

  None
}

/// Main entry point for module resolution.
// #[tracing::instrument(err, "resolve", skip_all, fields(
//     resolve.specifier = args.specifier,
//     resolve.importer = ?args.importer,
//     resolve.context = ?args.context,
//     resolve.dependency_type = ?args.dependency_type,
//     resolve.dependency_category = ?args.dependency_category
//   ),
//   level = "trace"
// )]
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
) -> Result<ResolveResult, Error> {
  let dep = ResolveOptionsWithDependencyType {
    resolve_options: args
      .resolve_options
      .clone()
      .map(|r| Box::new(Arc::unwrap_or_clone(r))),
    resolve_to_context: args.resolve_to_context,
    dependency_category: *args.dependency_category,
  };

  let mut context = Default::default();
  let resolver = plugin_driver.resolver_factory.get(dep);
  let mut result = resolver
    .resolve_with_context(args.context.as_ref(), args.specifier, &mut context)
    .await
    .map_err(|error| error.into_resolve_error(&args));

  if let Err(ref err) = result {
    tracing::error!(
      specifier = args.specifier,
      importer = ?args.importer,
      context = %args.context,
      dependency_type = %args.dependency_type,
      dependency_category = %args.dependency_category,
      "Resolve error: {}",
      err.to_string()
    );
  }

  args.file_dependencies.extend(context.file_dependencies);
  args
    .missing_dependencies
    .extend(context.missing_dependencies);

  if result.is_err()
    && let Some(hint) = resolve_for_error_hints(args, plugin_driver, resolver.inner_fs()).await
  {
    result = result.map_err(|mut err| {
      err.help = Some(hint);
      err
    })
  };

  result
}
