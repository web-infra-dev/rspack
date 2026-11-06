use cow_utils::CowUtils;
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{
  glob::{
    GlobDfa, GlobDfaState, GlobExpression, GlobMatchOptions, escape_glob_pattern,
    extract_glob_base_dir, glob_allows_dot_components, glob_match_normalized_with_explicit_dot,
    normalize_path_separators, normalize_path_separators_for_path, unescape_glob_path,
  },
  identifier::relative_path_to_request,
  node_path::NodePath,
};
use smallvec::SmallVec;
use sugar_path::SugarPath;

use crate::{ContextModuleOptions, ResolveInnerOptions};

#[derive(Debug)]
struct ContextModuleGlobPattern {
  pattern: String,
  pattern_base: String,
  negative: bool,
  root_relative: bool,
  dfa_index: Option<usize>,
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
  let matcher_pattern = Utf8Path::new(&matcher_pattern)
    .node_normalize_posix()
    .to_string();
  let matcher_pattern = if root_relative {
    matcher_pattern
  } else {
    relative_path_to_request(&matcher_pattern).into_owned()
  };
  let pattern_base = unescape_glob_path(extract_glob_base_dir(&matcher_pattern));

  ContextModuleGlobPattern {
    pattern: matcher_pattern,
    pattern_base,
    negative,
    root_relative,
    dfa_index: None,
  }
}

pub(super) struct ContextModuleGlobMatcher<'a> {
  patterns: Vec<ContextModuleGlobPattern>,
  dfa: Option<GlobDfa>,
  fallback_include: bool,
  positive_pattern_bases: Vec<String>,
  context: &'a str,
  compiler_context: &'a str,
  exhaustive: bool,
  case_sensitive: bool,
}

impl<'a> ContextModuleGlobMatcher<'a> {
  pub(super) fn new(options: &'a ContextModuleOptions) -> Option<Self> {
    let context_options = &options.context_options;
    let mut patterns = context_options
      .pattern
      .glob_patterns()?
      .iter()
      .map(|pattern| parse_context_module_glob_pattern(pattern))
      .collect::<Vec<_>>();
    let positive_pattern_bases = if context_options.glob_case_sensitive {
      Vec::new()
    } else {
      patterns
        .iter()
        .filter(|pattern| !pattern.negative)
        .map(|pattern| {
          absolute_context_module_glob_pattern_base(
            pattern,
            &context_options.context,
            &context_options.compiler_context,
          )
        })
        .collect()
    };
    let dfa = context_options
      .glob_case_sensitive
      .then(|| {
        compile_context_glob_dfa(
          &mut patterns,
          &context_options.context,
          &context_options.compiler_context,
          &options.resource,
        )
      })
      .flatten();
    let fallback_include = patterns
      .iter()
      .any(|pattern| !pattern.negative && pattern.dfa_index.is_none());
    Some(Self {
      patterns,
      dfa,
      fallback_include,
      positive_pattern_bases,
      context: &context_options.context,
      compiler_context: &context_options.compiler_context,
      exhaustive: context_options.glob_exhaustive,
      case_sensitive: context_options.glob_case_sensitive,
    })
  }

  pub(super) fn is_empty(&self) -> bool {
    self.patterns.is_empty()
  }

  pub(super) fn scan_state(&self, root: &Utf8Path) -> Option<GlobDfaState<'_>> {
    let root = normalize_path_separators_for_path(root.node_normalize().as_str());
    let state = self.dfa.as_ref()?.start().advance(&root);
    Some(if root.ends_with('/') {
      state
    } else {
      state.advance("/")
    })
  }

  pub(super) fn can_visit(&self, state: GlobDfaState<'_>) -> bool {
    self.fallback_include || state.can_include()
  }

  pub(super) fn match_request(
    &self,
    path: &str,
    state: Option<GlobDfaState<'_>>,
  ) -> Option<String> {
    let mut accepted = SmallVec::<[usize; 4]>::new();
    if let Some(state) = state {
      accepted.extend(state.matches());
    }
    let match_pattern = |pattern: &ContextModuleGlobPattern| {
      if let Some(index) = pattern.dfa_index
        && !accepted.contains(&index)
      {
        return None;
      }
      let request = context_relative_glob_request(
        path,
        if pattern.root_relative {
          self.compiler_context
        } else {
          self.context
        },
        pattern.root_relative,
      );
      let matched = if pattern.dfa_index.is_some() {
        glob_allows_dot_components(
          &pattern.pattern,
          &request,
          &pattern.pattern_base,
          &GlobMatchOptions {
            case_sensitive: true,
            require_literal_leading_dot: !self.exhaustive,
          },
        )
      } else {
        glob_pattern_matches(pattern, &request, self.exhaustive, self.case_sensitive)
      };
      matched.then_some(request)
    };
    let user_request = self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .find_map(match_pattern)?;
    if self
      .patterns
      .iter()
      .filter(|pattern| pattern.negative)
      .any(|pattern| match_pattern(pattern).is_some())
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

fn compile_context_glob_dfa(
  patterns: &mut [ContextModuleGlobPattern],
  context: &str,
  compiler_context: &str,
  scan_root: &Utf8Path,
) -> Option<GlobDfa> {
  let literal_directory = |path: &str| {
    let mut path =
      normalize_path_separators_for_path(Utf8Path::new(path).node_normalize().as_str());
    if !path.ends_with('/') {
      path.push('/');
    }
    path
  };
  let importer = literal_directory(context);
  let project = literal_directory(compiler_context);
  let scan_is_inside_project = scan_root
    .node_normalize()
    .starts_with(Utf8Path::new(compiler_context).node_normalize());
  let mut expressions = Vec::new();
  let mut indices = Vec::new();
  let mut include_count = 0;
  for negative in [false, true] {
    for (index, pattern) in patterns
      .iter()
      .enumerate()
      .filter(|(_, pattern)| pattern.negative == negative)
    {
      let (directory, suffix) = if pattern.root_relative {
        // When the scan reaches outside the project, a root-relative request
        // can start with /../ and cannot be represented by the project prefix.
        if !scan_is_inside_project {
          continue;
        }
        (&project, pattern.pattern.strip_prefix('/'))
      } else {
        // A canonical importer-relative path beginning with ../ changes
        // coordinates when the walk enters the importer's own subtree.
        (&importer, pattern.pattern.strip_prefix("./"))
      };
      if let Some(expression) = suffix.and_then(|suffix| GlobExpression::new(directory, suffix)) {
        indices.push(index);
        expressions.push(expression);
        include_count += usize::from(!negative);
      }
    }
  }
  if expressions.is_empty() {
    return None;
  }
  let dfa = GlobDfa::new(&expressions, include_count)?;
  for (dfa_index, pattern_index) in indices.into_iter().enumerate() {
    patterns[pattern_index].dfa_index = Some(dfa_index);
  }
  Some(dfa)
}

fn absolute_context_module_glob_pattern_base(
  pattern: &ContextModuleGlobPattern,
  context: &str,
  compiler_context: &str,
) -> String {
  let context = if pattern.root_relative {
    compiler_context
  } else {
    context
  };
  let pattern_base = pattern
    .pattern_base
    .strip_prefix('/')
    .unwrap_or(&pattern.pattern_base);
  let pattern_base = Utf8Path::new(&normalize_path_separators_for_path(context))
    .node_join_posix(pattern_base)
    .node_normalize_posix()
    .to_string();
  normalize_case_insensitive_path(&pattern_base)
}

fn normalize_case_insensitive_path(path: &str) -> String {
  let normalized_path = normalize_path_separators_for_path(path);
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
  if !case_sensitive {
    let pattern_value = pattern.pattern.cow_to_lowercase();
    let path = normalized_path.cow_to_lowercase();
    let pattern_base = pattern.pattern_base.cow_to_lowercase();
    return glob_match_normalized_with_explicit_dot(
      &pattern_value,
      &path,
      &pattern_base,
      &GlobMatchOptions {
        case_sensitive: true,
        require_literal_leading_dot: !exhaustive,
      },
    );
  }

  glob_match_normalized_with_explicit_dot(
    &pattern.pattern,
    normalized_path,
    &pattern.pattern_base,
    &GlobMatchOptions {
      case_sensitive: true,
      require_literal_leading_dot: !exhaustive,
    },
  )
}
