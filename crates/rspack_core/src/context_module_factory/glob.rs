use cow_utils::CowUtils;
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{identifier::relative_path_to_request, node_path::NodePath};
use sugar_path::SugarPath;

use crate::{
  ContextModuleOptions, GlobMatchOptions, escape_glob_pattern, extract_glob_base_dir,
  glob_match_normalized_with_explicit_dot, normalize_path_separators,
  normalize_path_separators_for_path, unescape_glob_path,
};

#[derive(Debug)]
struct ContextModuleGlobPattern {
  pattern: String,
  pattern_base: String,
  negative: bool,
  root_relative: bool,
}

#[derive(Debug)]
struct ResolvedContextModuleGlobPattern {
  absolute_pattern: String,
  absolute_base: String,
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
    .map(|pattern| resolve_context_module_glob_pattern(pattern, context, compiler_context))
    .collect::<Vec<_>>();
  let common_base = if case_sensitive {
    common_context_module_glob_base(&resolved_patterns)
  } else {
    case_insensitive_context_module_glob_base(patterns, context, compiler_context)
  };
  let Some(common_base) = common_base else {
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

fn case_insensitive_context_module_glob_base(
  patterns: &[String],
  context: &str,
  compiler_context: &str,
) -> Option<Utf8PathBuf> {
  let roots = patterns
    .iter()
    .map(|pattern| parse_context_module_glob_pattern(pattern))
    .filter(|pattern| !pattern.negative)
    .map(|pattern| {
      let (base, pattern) = if pattern.root_relative {
        (
          compiler_context,
          pattern
            .pattern
            .strip_prefix('/')
            .unwrap_or(&pattern.pattern),
        )
      } else {
        (context, pattern.pattern.as_str())
      };
      let normalized_pattern = Utf8Path::new(pattern).node_normalize_posix().to_string();
      let stable_prefix = normalized_pattern
        .split('/')
        .take_while(|segment| segment.is_empty() || *segment == "." || *segment == "..")
        .collect::<Vec<_>>()
        .join("/");
      Utf8Path::new(&normalize_path_separators_for_path(base))
        .node_join_posix(&stable_prefix)
        .node_normalize_posix()
    })
    .collect::<Vec<_>>();

  common_path_base(roots.iter().map(Utf8PathBuf::as_path))
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
      .map(|pattern| Utf8Path::new(pattern.absolute_base.as_str())),
  )
}

fn resolve_context_module_glob_pattern(
  pattern: &str,
  context: &str,
  compiler_context: &str,
) -> ResolvedContextModuleGlobPattern {
  let pattern = parse_context_module_glob_pattern(pattern);
  let (base, pattern_to_join) = if pattern.root_relative {
    (
      compiler_context,
      pattern
        .pattern
        .strip_prefix('/')
        .unwrap_or(pattern.pattern.as_str()),
    )
  } else {
    (context, pattern.pattern.as_str())
  };
  let base = normalize_path_separators_for_path(base);
  let escaped_base = escape_glob_pattern(&base);
  let absolute_pattern = Utf8Path::new(&escaped_base)
    .node_join_posix(pattern_to_join)
    .node_normalize_posix()
    .to_string();
  let absolute_pattern = normalize_path_separators(&absolute_pattern);
  let absolute_base = unescape_glob_path(extract_glob_base_dir(&absolute_pattern));

  ResolvedContextModuleGlobPattern {
    absolute_pattern,
    absolute_base,
    negative: pattern.negative,
  }
}

fn glob_patterns_are_recursive(
  patterns: &[ResolvedContextModuleGlobPattern],
  common_base: &Utf8Path,
) -> bool {
  patterns
    .iter()
    .filter(|pattern| !pattern.negative)
    .any(|pattern| {
      pattern.absolute_pattern.contains("**")
        || pattern
          .absolute_pattern
          .strip_prefix(common_base.as_str())
          .unwrap_or(pattern.absolute_pattern.as_str())
          .contains('/')
    })
}

fn parse_context_module_glob_pattern(pattern: &str) -> ContextModuleGlobPattern {
  let (pattern, negative) = if let Some(pattern) = pattern.strip_prefix('!') {
    (pattern, true)
  } else {
    (pattern, false)
  };
  let pattern = normalize_path_separators(pattern);
  let root_relative = pattern.starts_with('/');
  let matcher_pattern = if root_relative || pattern.starts_with("./") || pattern.starts_with("../")
  {
    pattern
  } else {
    relative_path_to_request(&pattern).into_owned()
  };
  let pattern_base = unescape_glob_path(extract_glob_base_dir(&matcher_pattern));

  ContextModuleGlobPattern {
    pattern: matcher_pattern,
    pattern_base,
    negative,
    root_relative,
  }
}

pub(super) struct ContextModuleGlobMatcher<'a> {
  patterns: Vec<ContextModuleGlobPattern>,
  context: &'a str,
  compiler_context: &'a str,
  exhaustive: bool,
  case_sensitive: bool,
}

impl<'a> ContextModuleGlobMatcher<'a> {
  pub(super) fn new(options: &'a ContextModuleOptions) -> Option<Self> {
    let context_options = &options.context_options;
    let patterns = context_options
      .pattern
      .glob_patterns()?
      .iter()
      .map(|pattern| parse_context_module_glob_pattern(pattern))
      .collect();
    Some(Self {
      patterns,
      context: &context_options.context,
      compiler_context: &context_options.compiler_context,
      exhaustive: context_options.glob_exhaustive,
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
        glob_pattern_matches(pattern, &request, self.exhaustive, self.case_sensitive)
          .then_some(request)
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
        glob_pattern_matches(pattern, &request, self.exhaustive, self.case_sensitive)
      })
    {
      return None;
    }

    Some(user_request)
  }

  pub(super) fn should_visit_skipped_dir(&self, path: &str) -> bool {
    if self.case_sensitive {
      return false;
    }

    self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
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
        let request = request.trim_end_matches('/').cow_to_lowercase();
        let pattern_base = pattern
          .pattern_base
          .trim_end_matches('/')
          .cow_to_lowercase();
        pattern_base == request || pattern_base.starts_with(&format!("{request}/"))
      })
  }
}

fn context_relative_glob_request(path: &str, context: &str, root_relative: bool) -> String {
  let relative_path = Utf8Path::new(path).as_std_path().relative(context);
  let relative_path = normalize_path_separators_for_path(&relative_path.to_string_lossy());
  if root_relative {
    format!("/{}", relative_path.trim_start_matches('/'))
  } else {
    relative_path_to_request(&relative_path).into_owned()
  }
}

fn glob_pattern_matches(
  pattern: &ContextModuleGlobPattern,
  normalized_path: &str,
  exhaustive: bool,
  case_sensitive: bool,
) -> bool {
  glob_match_normalized_with_explicit_dot(
    &pattern.pattern,
    normalized_path,
    &pattern.pattern_base,
    &GlobMatchOptions {
      case_sensitive,
      require_literal_leading_dot: !exhaustive,
    },
  )
}
