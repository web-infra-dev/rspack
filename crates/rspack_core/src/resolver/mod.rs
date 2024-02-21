mod factory;
mod resolver_impl;
use std::borrow::Borrow;
use std::fs;
use std::{fmt, path::PathBuf};

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

  // try to resolve relative path with extension
  if RELATIVE_PATH_REGEX.is_match(args.specifier) {
    let connected_path = base_dir.join(args.specifier);
    let normalized_path = connected_path.absolutize();

    let mut resolve_dir = false;

    let file_name = normalized_path.file_name();

    let parent_path = match fs::metadata(&normalized_path) {
      Ok(metadata) => {
        if !metadata.is_dir() {
          normalized_path.parent()
        } else {
          resolve_dir = true;
          Some(normalized_path.borrow())
        }
      }
      Err(_) => normalized_path.parent(),
    };

    if parent_path.is_none() || file_name.is_none() {
      return None;
    }

    let file_name = file_name.unwrap();
    let parent_path = parent_path.unwrap();

    let mut possible_matched_files = vec![file_name
      .to_str()
      .map(|f| f.to_string())
      .unwrap_or_default()];
    if resolve_dir {
      // also need to resolve the main files(like `index`) in the directory
      let main_files = dep
        .resolve_options
        .as_deref()
        .or(Some(&plugin_driver.options.resolve))
        .and_then(|o| o.main_files.as_ref().map(|f| f.clone()))
        .unwrap_or_else(|| Vec::new());

      possible_matched_files.extend(main_files);
    }

    // read the files in the parent directory
    let files = fs::read_dir(parent_path);
    match files {
      Ok(files) => {
        let suggestions = files
          .into_iter()
          .filter_map(|file| {
            file.ok().and_then(|file| {
              file.path().file_stem().and_then(|file_stem| {
                if possible_matched_files.contains(&file_stem.to_string_lossy().to_string()) {
                  let mut suggestion = file.path().relative(args.context.as_path());

                  if !suggestion.to_string_lossy().starts_with(".") {
                    suggestion = PathBuf::from(format!("./{}", suggestion.to_string_lossy()));
                  }
                  Some(suggestion)
                } else {
                  None
                }
              })
            })
          })
          .collect::<Vec<_>>();

        if suggestions.len() == 0 {
          return None;
        }

        let mut hint: Vec<String> = vec![];
        for suggestion in suggestions {
          let suggestion_ext = suggestion
            .extension()
            .map(|e| e.to_string_lossy())
            .unwrap_or_default();
          let suggestion_path = suggestion.to_string_lossy();
          let specifier = args.specifier;

          hint.push(format!(
          "Found the module '{suggestion_path}' exists, but it was not resolved because its extension doesn't in the `resolve.extensions` list. Here are some solutions:

1. use '{suggestion_path}' instead of '{specifier}'
2. add the extension '.{suggestion_ext}' to `resolve.extensions` in your rspack configuration"));
        }

        return Some(hint.join("\n"));
      }
      Err(_) => return None,
    }
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
