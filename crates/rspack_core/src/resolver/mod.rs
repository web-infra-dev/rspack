mod factory;
mod resolver_impl;

use std::{fmt, path::PathBuf};

use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::{Error, MietteExt};
use rspack_loader_runner::DescriptionData;
use sugar_path::{AsPath, SugarPath};

pub use self::factory::{ResolveOptionsWithDependencyType, ResolverFactory};
pub use self::resolver_impl::{ResolveInnerOptions, Resolver};
use crate::{ResolveArgs, SharedPluginDriver};

static RELATIVE_PATH_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\.\.?\/").expect("should init regex"));

static PARENT_PATH_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\.\.[\/]").expect("should init regex"));

static CURRENT_DIR_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(\.[\/])").expect("should init regex"));

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
  pub path: PathBuf,
  pub query: Option<String>,
  pub fragment: Option<String>,
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

pub fn resolve_for_error_hints(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
) -> Option<String> {
  let dep = ResolveOptionsWithDependencyType {
    resolve_options: args.resolve_options.clone(),
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
    match resolver.resolve(base_dir, args.specifier) {
      Ok(ResolveResult::Resource(resource)) => {
        let relative_path = resource.path.relative(args.context.as_path());
        let suggestion = if let Some((_, [prefix])) = CURRENT_DIR_REGEX
          .captures_iter(args.specifier)
          .next()
          .map(|c| c.extract())
        {
          // If the specifier is a relative path pointing to the current directory,
          // we can suggest the path relative to the current directory.
          format!("{}{}", prefix, relative_path.to_string_lossy())
        } else if PARENT_PATH_REGEX.is_match(args.specifier) {
          // If the specifier is a relative path to which the parent directory is,
          // then we return the relative path directly.
          relative_path.to_string_lossy().to_string()
        } else {
          // If the specifier is a package name like or some arbitrary alias,
          // then we return the full path.
          resource.path.to_string_lossy().to_string()
        };
        return Some(format!("Did you mean '{}'?

The request '{}' failed to resolve only because it was resolved as fully specified,
probably because the origin is strict EcmaScript Module,
e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\"type\": \"module\"'.

The extension in the request is mandatory for it to be fully specified.
Add the extension to the request.", suggestion, args.specifier));
      }
      Err(_) => return None,
      _ => {}
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
    match resolver.resolve(base_dir, &request) {
      Ok(ResolveResult::Resource(_)) => {
        return Some(format!(
          "Did you mean './{}'?

Requests that should resolve in the current directory need to start with './'.
Requests that start with a name are treated as module requests and resolve within module directories{module_directories}

If changing the source code is not an option, there is also a resolve options called 'preferRelative'
which tries to resolve these kind of requests in the current directory too.",
          args.specifier
        ));
      }
      Err(_) => return None,
      _ => {}
    }
  }

  if args.missing_dependencies.len() > 0 {
    let description_data = args
      .missing_dependencies
      .iter()
      .find(|p| p.ends_with("package.json"));

    let missing_dependencies = args
      .missing_dependencies
      .iter()
      .filter(|p| !p.ends_with("package.json"))
      .map(|p| {
        let path = p.to_string_lossy().to_string();
        format!("'{}' doesn't exist", path)
      })
      .collect::<Vec<_>>()
      .join("\n")
      .red();

    let using_description_data_hint = if let Some(description_data) = description_data {
      format!(
        "use description file: {}\n",
        description_data.to_string_lossy()
      )
    } else {
      "".to_string()
    };

    return Some(format!(
      "try to resolve '{}' in '{}'\n {}{}",
      args.specifier,
      base_dir.to_string_lossy(),
      using_description_data_hint,
      missing_dependencies
    ));
  }

  None
}

/// Main entry point for module resolution.
pub async fn resolve(
  args: ResolveArgs<'_>,
  plugin_driver: &SharedPluginDriver,
) -> Result<ResolveResult, Error> {
  let dep = ResolveOptionsWithDependencyType {
    resolve_options: args.resolve_options.clone(),
    resolve_to_context: args.resolve_to_context,
    dependency_category: *args.dependency_category,
  };

  let mut context = Default::default();
  let resolver = plugin_driver.resolver_factory.get(dep);
  let mut result = resolver
    .resolve_with_context(args.context.as_ref(), args.specifier, &mut context)
    .map_err(|error| error.into_resolve_error(&args));

  args.file_dependencies.extend(context.file_dependencies);
  args
    .missing_dependencies
    .extend(context.missing_dependencies);

  if result.is_err()
    && let Some(hint) = resolve_for_error_hints(args, plugin_driver)
  {
    result = result.map_err(|err| err.with_help(hint))
  };

  result.map_err(Error::new_boxed)
}
