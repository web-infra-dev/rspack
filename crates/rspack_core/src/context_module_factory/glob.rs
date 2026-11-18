use cow_utils::CowUtils;
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf, normalize_path_separators};
use rspack_util::{
  glob::{GlobOptions, GlobPattern, normalize_glob_path},
  identifier::relative_path_to_request,
  node_path::NodePath,
};
use sugar_path::SugarPath;

use crate::{ContextModuleOptions, ResolveInnerOptions};

struct ContextModuleGlobPattern {
  pattern: String,
  negative: bool,
  root_relative: bool,
}

impl ContextModuleGlobPattern {
  fn base<'a>(&self, context: &'a str, compiler_context: &'a str) -> &'a str {
    if self.root_relative {
      compiler_context
    } else {
      context
    }
  }
}

struct ResolvedContextModuleGlobPattern {
  absolute_base: Utf8PathBuf,
  scan_base: Utf8PathBuf,
  recursive: bool,
  negative: bool,
}

#[derive(Debug)]
pub struct CompiledContextModuleGlobRequest {
  pub request: String,
  pub recursive: bool,
}

pub fn compile_context_module_glob_request(
  request: &str,
  patterns: &[String],
  context: &str,
  compiler_context: &str,
  fallback_recursive: bool,
  case_sensitive: bool,
) -> CompiledContextModuleGlobRequest {
  let Some(parsed_request) = parse_resource(request) else {
    return CompiledContextModuleGlobRequest {
      request: request.to_string(),
      recursive: fallback_recursive,
    };
  };
  let resolved_patterns = patterns
    .iter()
    .filter_map(|pattern| {
      resolve_context_module_glob_pattern(pattern, context, compiler_context, case_sensitive)
    })
    .collect::<Vec<_>>();
  let Some(common_base) = common_context_module_glob_base(&resolved_patterns) else {
    return CompiledContextModuleGlobRequest {
      request: request.to_string(),
      recursive: fallback_recursive,
    };
  };

  let recursive = glob_patterns_are_recursive(&resolved_patterns, &common_base);
  let mut request = context_relative_glob_request(common_base.as_str(), context, false);
  if request.ends_with("/.") {
    request.pop();
  }
  if let Some(query) = parsed_request.query {
    request.push_str(&query);
  }
  if let Some(fragment) = parsed_request.fragment {
    request.push_str(&fragment);
  }
  CompiledContextModuleGlobRequest { request, recursive }
}

/// Bare globs such as `@/dir/*.js` compile to `./@/dir/` so that they match
/// relative to the importer. Returns the request without `./` when its prefix
/// names a `resolve.alias` key, so the resolver can apply the alias.
pub(super) fn bare_context_module_glob_alias_request<'a>(
  patterns: &[String],
  request: &'a str,
  resolve_options: &ResolveInnerOptions,
) -> Option<&'a str> {
  let bare = patterns
    .iter()
    .filter(|pattern| !pattern.starts_with('!'))
    .all(|pattern| !pattern.starts_with('.') && !pattern.starts_with('/'));
  if !bare {
    return None;
  }
  let request = request.strip_prefix("./")?;
  let path = parse_resource(request)?.path;
  (!path.as_str().is_empty() && resolve_options.has_alias_for(path.as_str())).then_some(request)
}

pub(super) fn resolve_context_module_glob_alias(
  patterns: &[String],
  request: &str,
  resolved_base: &str,
  context: &str,
  compiler_context: &str,
) -> Vec<String> {
  let request = request.trim_end_matches('/');
  patterns
    .iter()
    .map(|pattern| {
      let (negative, path) = pattern
        .strip_prefix('!')
        .map_or(("", pattern.as_str()), |path| ("!", path));
      let Some(suffix) = path
        .strip_prefix(request)
        .and_then(|path| path.strip_prefix('/'))
      else {
        return pattern.clone();
      };
      let resolved_pattern = Utf8Path::new(resolved_base)
        .node_join_posix(suffix)
        .node_normalize_posix();
      let resolved_pattern = resolved_pattern.as_str();
      let rewritten = if Utf8Path::new(resolved_pattern).starts_with(compiler_context) {
        context_relative_glob_request(resolved_pattern, compiler_context, true)
      } else {
        context_relative_glob_request(resolved_pattern, context, false)
      };
      format!("{negative}{rewritten}")
    })
    .collect()
}

fn common_path_base<'a>(mut paths: impl Iterator<Item = &'a Utf8Path>) -> Option<Utf8PathBuf> {
  let mut common_base = paths.next()?.to_path_buf();
  for path in paths {
    while !path.starts_with(&common_base) {
      common_base = common_base.parent()?.to_path_buf();
    }
  }
  Some(common_base)
}

fn common_context_module_glob_base(
  patterns: &[ResolvedContextModuleGlobPattern],
) -> Option<Utf8PathBuf> {
  common_path_base(
    patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .map(|pattern| pattern.scan_base.as_path()),
  )
}

fn resolve_context_module_glob_pattern(
  pattern: &str,
  context: &str,
  compiler_context: &str,
  case_sensitive: bool,
) -> Option<ResolvedContextModuleGlobPattern> {
  let source = parse_context_module_glob_pattern(pattern);
  let matcher = GlobPattern::new_with_options(
    source.pattern.as_bytes(),
    GlobOptions {
      case_sensitive,
      ..Default::default()
    },
  )
  .ok()?;
  let context = source.base(context, compiler_context);
  let prefix = matcher.directory_prefix();
  let absolute_base = resolve_pattern_base(context, prefix);
  let scan_base = if case_sensitive {
    absolute_base.clone()
  } else {
    resolve_pattern_base(context, matcher.scan_directory_prefix().as_bytes())
  };
  let recursive = matcher
    .match_prefix(prefix)
    .is_some_and(|remaining| remaining.is_recursive());
  Some(ResolvedContextModuleGlobPattern {
    absolute_base,
    scan_base,
    recursive,
    negative: source.negative,
  })
}

fn resolve_pattern_base(context: &str, prefix: &[u8]) -> Utf8PathBuf {
  let prefix = String::from_utf8_lossy(prefix);
  Utf8Path::new(&normalize_path_separators(context))
    .node_join_posix(prefix.trim_start_matches('/'))
    .node_normalize_posix()
}

fn glob_patterns_are_recursive(
  patterns: &[ResolvedContextModuleGlobPattern],
  common_base: &Utf8Path,
) -> bool {
  patterns
    .iter()
    .filter(|pattern| !pattern.negative)
    .any(|pattern| pattern.recursive || pattern.absolute_base != common_base)
}

fn parse_context_module_glob_pattern(pattern: &str) -> ContextModuleGlobPattern {
  let (pattern, negative) = pattern
    .strip_prefix('!')
    .map_or((pattern, false), |pattern| (pattern, true));
  let pattern = normalize_glob_path(pattern);
  let root_relative = pattern.starts_with('/');
  let normalized = Utf8Path::new(pattern.as_ref()).node_normalize_posix();
  let pattern = if root_relative {
    normalized.into_string()
  } else {
    relative_path_to_request(normalized.as_str()).into_owned()
  };
  ContextModuleGlobPattern {
    pattern,
    negative,
    root_relative,
  }
}

struct ContextGlobMatch {
  matcher: GlobPattern<'static>,
  negative: bool,
  root_relative: bool,
}

pub(super) struct ContextModuleGlobMatcher<'a> {
  patterns: Vec<ContextGlobMatch>,
  positive_pattern_bases: Vec<String>,
  context: &'a str,
  compiler_context: &'a str,
  case_sensitive: bool,
}

impl<'a> ContextModuleGlobMatcher<'a> {
  pub(super) fn new(options: &'a ContextModuleOptions) -> Option<Self> {
    let context_options = &options.context_options;
    let glob_options = GlobOptions {
      case_sensitive: context_options.glob_case_sensitive,
      require_literal_leading_dot: !context_options.glob_exhaustive,
      ..Default::default()
    };
    let mut positive_pattern_bases = Vec::new();
    let patterns = context_options
      .pattern
      .glob_patterns()?
      .iter()
      .filter_map(|pattern| {
        let source = parse_context_module_glob_pattern(pattern);
        let matcher =
          GlobPattern::new_with_options(source.pattern.as_bytes(), glob_options).ok()?;
        if !glob_options.case_sensitive && !source.negative {
          let context = source.base(&context_options.context, &context_options.compiler_context);
          let base = resolve_pattern_base(context, matcher.directory_prefix());
          positive_pattern_bases.push(normalize_case_insensitive_path(base.as_str()));
        }
        Some(ContextGlobMatch {
          matcher: matcher.into_owned(),
          negative: source.negative,
          root_relative: source.root_relative,
        })
      })
      .collect();
    Some(Self {
      patterns,
      positive_pattern_bases,
      context: &context_options.context,
      compiler_context: &context_options.compiler_context,
      case_sensitive: context_options.glob_case_sensitive,
    })
  }

  pub(super) fn is_empty(&self) -> bool {
    self.patterns.is_empty()
  }

  pub(super) fn match_request(&self, path: &str) -> Option<String> {
    let user_request = self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .find_map(|pattern| {
        let request = context_relative_glob_request(
          path,
          if pattern.root_relative {
            self.compiler_context
          } else {
            self.context
          },
          pattern.root_relative,
        );
        pattern.matcher.is_match(&request).then_some(request)
      })?;

    if self
      .patterns
      .iter()
      .filter(|pattern| pattern.negative)
      .any(|pattern| {
        let request = context_relative_glob_request(
          path,
          if pattern.root_relative {
            self.compiler_context
          } else {
            self.context
          },
          pattern.root_relative,
        );
        pattern.matcher.is_match(&request)
      })
    {
      return None;
    }

    Some(user_request)
  }

  pub(super) fn should_visit_dir(&self, path: &str) -> bool {
    if self.case_sensitive {
      return true;
    }

    let path = normalize_case_insensitive_path(path);
    self.positive_pattern_bases.iter().any(|pattern_base| {
      is_same_or_descendant(pattern_base, &path) || is_same_or_descendant(&path, pattern_base)
    })
  }

  pub(super) fn should_visit_skipped_dir(&self, path: &str) -> bool {
    if self.case_sensitive {
      return false;
    }

    let path = normalize_case_insensitive_path(path);
    self
      .positive_pattern_bases
      .iter()
      .any(|pattern_base| is_same_or_descendant(pattern_base, &path))
  }
}

fn normalize_case_insensitive_path(path: &str) -> String {
  let normalized_path = normalize_path_separators(path);
  let path = normalized_path.trim_end_matches('/');
  if path.is_empty() {
    "/".to_string()
  } else {
    path.cow_to_lowercase().into_owned()
  }
}

fn is_same_or_descendant(path: &str, base: &str) -> bool {
  path == base
    || (base == "/" && path.starts_with('/'))
    || path
      .strip_prefix(base)
      .is_some_and(|suffix| suffix.starts_with('/'))
}

fn context_relative_glob_request(path: &str, context: &str, root_relative: bool) -> String {
  let relative_path = Utf8Path::new(path).as_std_path().relative(context);
  let relative_path = relative_path.to_string_lossy();
  let relative_path = normalize_path_separators(&relative_path);
  if root_relative {
    format!("/{}", relative_path.trim_start_matches('/'))
  } else {
    relative_path_to_request(&relative_path).into_owned()
  }
}
