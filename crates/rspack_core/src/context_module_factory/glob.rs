use cow_utils::CowUtils;
use rspack_glob::{
  GlobOptions, GlobPattern, extract_glob_base_dir, glob_base_dir_end, normalize_path_separators,
  normalize_path_separators_for_path, unescape_glob_path,
};
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{identifier::relative_path_to_request, node_path::NodePath};
use sugar_path::SugarPath;

use crate::{ContextGlobAlias, ContextGlobScan, ContextModuleOptions, ResolveInnerOptions};

#[derive(Debug)]
pub(super) struct ContextModuleGlobPattern {
  pattern: String,
  pattern_base: String,
  negative: bool,
  root_relative: bool,
}

#[derive(Debug)]
struct ResolvedContextModuleGlobPattern {
  pattern: String,
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

pub(super) struct ContextModuleGlobAliasRequest {
  pub request: String,
  pub prefix: String,
}

/// Find aliases before common-directory calculations discard literal prefixes.
pub(super) fn context_module_glob_alias_request(
  pattern: &str,
  resolve_options: &ResolveInnerOptions,
) -> Option<ContextModuleGlobAliasRequest> {
  let path = pattern.strip_prefix('!').unwrap_or(pattern);
  if path.starts_with('.') || path.starts_with('/') {
    return None;
  }
  let path = normalize_path_separators(path);
  let pattern_base = extract_glob_base_dir(&path);
  let base = unescape_glob_path(pattern_base);
  let (key, exact) = resolve_options.alias_prefix(&base, pattern_base == path)?;
  Some(ContextModuleGlobAliasRequest {
    // /., unlike a trailing /, cannot trigger an exact key such as dir/$.
    request: if exact {
      key.to_string()
    } else {
      format!("{key}/.")
    },
    prefix: format!("{}/", relative_path_to_request(key.trim_end_matches('/'))),
  })
}

/// Compute physical scan roots while preserving raw pattern syntax.
pub(super) fn compile_aliased_glob_request(
  request: &str,
  patterns: &[String],
  aliases: Vec<ContextGlobAlias>,
  context: &str,
  compiler_context: &str,
  case_sensitive: bool,
) -> Option<(CompiledContextModuleGlobRequest, ContextGlobScan)> {
  let mut resolved_patterns = Vec::new();
  for raw in patterns {
    let mut resolved = resolve_context_module_glob_pattern(raw, context, compiler_context);
    if let Some(alias) = aliases.iter().find(|alias| alias.pattern == *raw) {
      let source = parse_context_module_glob_pattern(raw);
      let suffix = source
        .pattern_base
        .strip_prefix(&alias.prefix)
        .unwrap_or("");
      resolved.absolute_base = if case_sensitive {
        alias
          .resource
          .node_join_posix(suffix)
          .node_normalize_posix()
          .to_string()
      } else {
        alias.resource.to_string()
      };
    } else if !case_sensitive && !resolved.negative {
      resolved.absolute_base = case_insensitive_context_module_glob_base(
        std::slice::from_ref(raw),
        context,
        compiler_context,
      )?
      .to_string();
    }
    resolved_patterns.push(resolved);
  }
  let root = common_context_module_glob_base(&resolved_patterns)?;
  let recursive = glob_patterns_are_recursive(&resolved_patterns, &root)
    || (!case_sensitive
      && aliases.iter().any(|alias| {
        !alias.pattern.starts_with('!')
          && parse_context_module_glob_pattern(&alias.pattern)
            .pattern_base
            .strip_prefix(&alias.prefix)
            .is_some_and(|suffix| !suffix.is_empty())
      }));
  let mut scan_request = context_relative_glob_request(root.as_str(), context, false);
  if let Some(parsed) = parse_resource(request) {
    if let Some(query) = parsed.query {
      scan_request.push_str(&query);
    }
    if let Some(fragment) = parsed.fragment {
      scan_request.push_str(&fragment);
    }
  }
  Some((
    CompiledContextModuleGlobRequest {
      request: scan_request,
      recursive,
    },
    ContextGlobScan { root, aliases },
  ))
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
  let literal_base = unescape_glob_path(extract_glob_base_dir(pattern_to_join));
  let absolute_base = Utf8Path::new(&base)
    .node_join_posix(&literal_base)
    .node_normalize_posix()
    .to_string();
  ResolvedContextModuleGlobPattern {
    pattern: pattern.pattern,
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
      pattern.pattern.contains("**")
        || Utf8Path::new(&pattern.absolute_base) != common_base
        || pattern.pattern[glob_base_dir_end(&pattern.pattern)..].contains('/')
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
  }
}

struct CompiledGlobPattern<'a> {
  matcher: GlobPattern<'a>,
  negative: bool,
  root_relative: bool,
  alias: Option<Utf8PathBuf>,
  query: String,
  fragment: String,
  absolute_base: String,
}

pub(super) struct ContextModuleGlobMatcher<'a> {
  patterns: Vec<CompiledGlobPattern<'a>>,
  context: &'a str,
  compiler_context: &'a str,
  case_sensitive: bool,
}

pub(super) fn parse_context_module_glob_patterns(
  options: &ContextModuleOptions,
) -> Option<Vec<ContextModuleGlobPattern>> {
  Some(
    options
      .context_options
      .pattern
      .glob_patterns()?
      .iter()
      .map(|pattern| parse_context_module_glob_pattern(pattern))
      .collect(),
  )
}

impl<'a> ContextModuleGlobMatcher<'a> {
  pub(super) fn new(
    options: &'a ContextModuleOptions,
    sources: &'a [ContextModuleGlobPattern],
  ) -> Option<Self> {
    let context_options = &options.context_options;
    let patterns = context_options
      .pattern
      .glob_patterns()?
      .iter()
      .zip(sources)
      .filter_map(|(raw, source)| {
        let mut matcher = GlobPattern::new_with_options(
          source.pattern.as_bytes(),
          GlobOptions {
            case_sensitive: context_options.glob_case_sensitive,
            require_literal_leading_dot: !context_options.glob_exhaustive,
          },
        )
        .ok()?;
        let alias = context_options.glob_alias.as_ref().and_then(|scan| {
          scan
            .aliases
            .iter()
            .find(|alias| alias.pattern == *raw)
            .map(|alias| (scan, alias))
        });
        let (alias_root, absolute_base) = if let Some((scan, alias)) = alias {
          matcher = matcher.match_prefix(&alias.prefix)?;
          // Relocation applies to the physical tree, never to the glob syntax.
          let offset = alias.resource.as_std_path().relative(&scan.root);
          let root = options
            .resource
            .node_join_posix(offset.to_string_lossy().as_ref())
            .node_normalize_posix();
          let suffix = source
            .pattern_base
            .strip_prefix(&alias.prefix)
            .unwrap_or("");
          let base = normalize_case_insensitive_path(root.node_join_posix(suffix).as_str());
          (Some(root), base)
        } else {
          (
            None,
            absolute_context_module_glob_pattern_base(
              source,
              &context_options.context,
              &context_options.compiler_context,
            ),
          )
        };
        Some(CompiledGlobPattern {
          matcher,
          negative: source.negative,
          root_relative: alias_root.as_ref().map_or(source.root_relative, |root| {
            root.starts_with(context_options.compiler_context.as_path())
          }),
          alias: alias_root,
          query: alias
            .filter(|(_, alias)| !alias.query.is_empty())
            .map_or_else(
              || options.resource_query.clone(),
              |(_, alias)| alias.query.clone(),
            ),
          fragment: alias
            .filter(|(_, alias)| !alias.fragment.is_empty())
            .map_or_else(
              || options.resource_fragment.clone(),
              |(_, alias)| alias.fragment.clone(),
            ),
          absolute_base,
        })
      })
      .collect();
    Some(Self {
      patterns,
      context: &context_options.context,
      compiler_context: &context_options.compiler_context,
      case_sensitive: context_options.glob_case_sensitive,
    })
  }

  pub(super) fn is_empty(&self) -> bool {
    self.patterns.is_empty()
  }

  fn request(&self, pattern: &CompiledGlobPattern<'_>, path: &str) -> String {
    context_relative_glob_request(
      path,
      if pattern.root_relative {
        self.compiler_context
      } else {
        self.context
      },
      pattern.root_relative,
    )
  }

  fn matching_path(&self, pattern: &CompiledGlobPattern<'_>, path: &str) -> String {
    if let Some(root) = &pattern.alias {
      let relative = Utf8Path::new(path).as_std_path().relative(root);
      normalize_path_separators_for_path(&relative.to_string_lossy())
    } else {
      self.request(pattern, path)
    }
  }

  pub(super) fn match_request(&self, path: &str) -> Option<super::ContextModuleMatchedRequest<'_>> {
    let pattern = self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .find(|pattern| {
        pattern
          .matcher
          .match_path(self.matching_path(pattern, path))
          .is_exact()
      })?;
    if self
      .patterns
      .iter()
      .filter(|pattern| pattern.negative)
      .any(|pattern| {
        pattern
          .matcher
          .match_path(self.matching_path(pattern, path))
          .is_exact()
      })
    {
      return None;
    }
    Some(super::ContextModuleMatchedRequest {
      request: self.request(pattern, path),
      query: &pattern.query,
      fragment: &pattern.fragment,
    })
  }

  pub(super) fn should_visit_dir(&self, path: &str) -> bool {
    self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .any(|pattern| {
        // The scan root may be above an alias root in mixed-pattern globs.
        // Traversing towards that literal root does not consume the residual glob.
        if pattern
          .alias
          .as_ref()
          .is_some_and(|root| root.starts_with(path))
        {
          return true;
        }
        let mut prefix = self.matching_path(pattern, path);
        if !prefix.ends_with('/') {
          prefix.push('/');
        }
        pattern.matcher.match_prefix(prefix).is_some()
      })
  }

  pub(super) fn should_visit_skipped_dir(&self, path: &str) -> bool {
    self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .any(|pattern| {
        pattern
          .alias
          .as_ref()
          .is_some_and(|root| root.starts_with(path))
          || (!self.case_sensitive
            && is_same_or_descendant(
              &pattern.absolute_base,
              &normalize_case_insensitive_path(path),
            ))
      })
  }
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
