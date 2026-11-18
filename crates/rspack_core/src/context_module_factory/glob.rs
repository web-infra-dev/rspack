use cow_utils::CowUtils;
use rspack_hash::{HashDigest, HashFunction, RspackHasher, hash_by_json};
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf, normalize_path_separators};
use rspack_util::{
  glob::{GlobOptions, GlobPattern, normalize_glob_path},
  identifier::relative_path_to_request,
  node_path::NodePath,
};
use rustc_hash::FxHashMap;
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

pub(super) struct ContextModuleGlobAliasRequest {
  pub request: String,
  pub prefix: String,
  pub directory_suffix: String,
  pub resolves_directory: bool,
  pub recursive: bool,
}

/// Find aliases before common-directory calculations discard literal prefixes.
pub(super) fn context_module_glob_alias_request(
  pattern: &str,
  resolve_options: &ResolveInnerOptions,
  case_sensitive: bool,
) -> Option<ContextModuleGlobAliasRequest> {
  let path = pattern.strip_prefix('!').unwrap_or(pattern);
  if path.starts_with('.') || path.starts_with('/') {
    return None;
  }
  let source = parse_context_module_glob_pattern(pattern);
  let matcher = GlobPattern::new(source.pattern.as_bytes()).ok()?;
  let directory_prefix = matcher.directory_prefix();
  let base = std::str::from_utf8(directory_prefix)
    .ok()?
    .strip_prefix("./")?;
  let allow_exact =
    matcher.literal_prefix() == directory_prefix && matcher.is_match(directory_prefix);
  let (key, exact) = resolve_options.alias_prefix(base, allow_exact)?;
  let key_path = key.trim_end_matches('/');
  let mut prefix = format!("./{key_path}/");
  let mut directory_suffix = base
    .strip_prefix(key_path)
    .and_then(|suffix| suffix.strip_prefix('/'))
    .unwrap_or_default()
    .to_string();
  // The resolver probes exact filesystem spellings and treats '?' as a query.
  // Keep those components in the matcher when they cannot be resolved literally.
  let resolves_directory =
    !exact && case_sensitive && !directory_suffix.is_empty() && !directory_suffix.contains('?');
  let request = if exact {
    key.to_string()
  } else if resolves_directory {
    // Resolve the directory inside the alias so the resolver can try each target.
    prefix.push_str(&directory_suffix);
    let request = format!(
      "{}/{}.",
      key.cow_replace('#', "\0#"),
      directory_suffix.cow_replace('#', "\0#")
    );
    directory_suffix.clear();
    request
  } else {
    // /., unlike a trailing /, cannot trigger an exact key such as dir/$.
    format!("{key}/.")
  };
  let recursive = matcher
    .match_prefix(&prefix)
    .is_some_and(|remaining| remaining.is_recursive());
  Some(ContextModuleGlobAliasRequest {
    request,
    prefix,
    directory_suffix,
    resolves_directory,
    recursive,
  })
}

pub(super) struct ResolvedGlobAlias {
  pub prefix: String,
  pub directory_suffix: String,
  pub recursive: bool,
  pub resource: Utf8PathBuf,
  pub query: String,
  pub fragment: String,
}

pub(super) struct ResolvedGlobScan {
  root: Utf8PathBuf,
  aliases: FxHashMap<String, ResolvedGlobAlias>,
}

pub(super) fn prepare_context_module_glob_matches(
  options: &mut ContextModuleOptions,
  scan: Option<ResolvedGlobScan>,
) {
  let context = &options.context_options;
  let Some(patterns) = context.pattern.glob_patterns() else {
    return;
  };
  let glob_options = GlobOptions {
    case_sensitive: context.glob_case_sensitive,
    require_literal_leading_dot: !context.glob_exhaustive,
    ..Default::default()
  };
  let mut key = None;
  let matches: Vec<_> = patterns
    .iter()
    .enumerate()
    .filter_map(|(index, raw)| {
      let source = parse_context_module_glob_pattern(raw);
      let matcher = GlobPattern::new_with_options(source.pattern.as_bytes(), glob_options).ok()?;
      let compute_base = !source.negative && !glob_options.case_sensitive;
      if let Some((scan, alias)) = scan
        .as_ref()
        .and_then(|scan| scan.aliases.get(raw).map(|alias| (scan, alias)))
      {
        // Preserve the offset within the scan tree if afterResolve moved its root.
        let matcher = matcher.match_prefix(&alias.prefix)?.into_owned();
        let offset = alias.resource.as_std_path().relative(&scan.root);
        let offset = normalize_path_separators(&offset.to_string_lossy()).into_owned();
        let root = options.resource.node_join(&offset).node_normalize();
        let case_insensitive_base = compute_base.then(|| {
          normalize_case_insensitive_path(root.node_join(&alias.directory_suffix).as_str())
        });
        let root_relative = root.starts_with(context.compiler_context.as_path());
        let query = nonempty_suffix(&alias.query);
        let fragment = nonempty_suffix(&alias.fragment);
        hash_by_json(
          &(
            index,
            &offset,
            &alias.prefix,
            &alias.directory_suffix,
            root_relative,
            query.unwrap_or(&options.resource_query),
            fragment.unwrap_or(&options.resource_fragment),
          ),
          key.get_or_insert_with(|| RspackHasher::new(&HashFunction::SHA256)),
        );
        return Some(ContextGlobMatch {
          matcher,
          match_root: Some(root),
          case_insensitive_base,
          negative: source.negative,
          root_relative,
          query: query.map(str::to_owned),
          fragment: fragment.map(str::to_owned),
        });
      }
      Some(ContextGlobMatch {
        match_root: None,
        case_insensitive_base: compute_base.then(|| {
          let base = source.base(&context.context, &context.compiler_context);
          let base = resolve_pattern_base(base, matcher.directory_prefix());
          normalize_case_insensitive_path(base.as_str())
        }),
        matcher: matcher.into_owned(),
        negative: source.negative,
        root_relative: source.root_relative,
        query: None,
        fragment: None,
      })
    })
    .collect();
  options.glob_matches = Some(matches.into());
  options.glob_matches_key = key.map(|hasher| hasher.digest(&HashDigest::Hex));
}

fn nonempty_suffix(suffix: &str) -> Option<&str> {
  (!suffix.is_empty()).then_some(suffix)
}

/// Compute physical scan roots while preserving raw pattern syntax.
pub(super) fn compile_aliased_glob_request(
  request: &str,
  patterns: &[String],
  aliases: FxHashMap<String, ResolvedGlobAlias>,
  context: &str,
  compiler_context: &str,
  resolve_context: &str,
  case_sensitive: bool,
) -> Option<(CompiledContextModuleGlobRequest, ResolvedGlobScan)> {
  let resolved_patterns = patterns
    .iter()
    .filter_map(|raw| match aliases.get(raw) {
      Some(alias) => Some(ResolvedContextModuleGlobPattern {
        absolute_base: alias.resource.clone(),
        scan_base: alias.resource.clone(),
        recursive: alias.recursive,
        negative: raw.starts_with('!'),
      }),
      None => resolve_context_module_glob_pattern(raw, context, compiler_context, case_sensitive),
    })
    .collect::<Vec<_>>();
  let root = common_context_module_glob_base(&resolved_patterns)?;
  let recursive = glob_patterns_are_recursive(&resolved_patterns, &root);
  let mut scan_request = context_relative_glob_request(root.as_str(), resolve_context, false);
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
    ResolvedGlobScan { root, aliases },
  ))
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

/// An owned matcher and its coordinates after the factory hooks.
#[derive(Debug)]
pub(crate) struct ContextGlobMatch {
  matcher: GlobPattern<'static>,
  match_root: Option<Utf8PathBuf>,
  case_insensitive_base: Option<String>,
  negative: bool,
  root_relative: bool,
  query: Option<String>,
  fragment: Option<String>,
}

#[derive(Clone, Copy)]
pub(super) struct ContextModuleGlobMatcher<'a> {
  patterns: &'a [ContextGlobMatch],
  context: &'a str,
  compiler_context: &'a str,
  case_sensitive: bool,
  query: &'a str,
  fragment: &'a str,
}

impl<'a> ContextModuleGlobMatcher<'a> {
  pub(super) fn new(options: &'a ContextModuleOptions) -> Option<Self> {
    let context_options = &options.context_options;
    Some(Self {
      patterns: options.glob_matches.as_deref()?,
      context: &context_options.context,
      compiler_context: &context_options.compiler_context,
      case_sensitive: context_options.glob_case_sensitive,
      query: &options.resource_query,
      fragment: &options.resource_fragment,
    })
  }

  pub(super) fn is_empty(&self) -> bool {
    self.patterns.is_empty()
  }

  fn request(&self, pattern: &ContextGlobMatch, path: &str) -> String {
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

  fn matching_path(&self, pattern: &ContextGlobMatch, path: &str) -> Option<String> {
    if let Some(root) = &pattern.match_root {
      let path = Utf8Path::new(path);
      if !path.starts_with(root) {
        return None;
      }
      let relative = path.as_std_path().relative(root);
      Some(normalize_path_separators(&relative.to_string_lossy()).into_owned())
    } else {
      Some(self.request(pattern, path))
    }
  }

  fn matches(&self, pattern: &ContextGlobMatch, path: &str) -> bool {
    self
      .matching_path(pattern, path)
      .is_some_and(|path| pattern.matcher.is_match(path))
  }

  pub(super) fn match_request(
    &self,
    path: &str,
    resource_request: String,
  ) -> Option<super::ContextModuleMatchedRequest<'a>> {
    let pattern = self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .find(|pattern| self.matches(pattern, path))?;
    if self
      .patterns
      .iter()
      .filter(|pattern| pattern.negative)
      .any(|pattern| self.matches(pattern, path))
    {
      return None;
    }
    Some(super::ContextModuleMatchedRequest {
      request: self.request(pattern, path),
      resource_request: Some(resource_request),
      query: pattern.query.as_deref().unwrap_or(self.query),
      fragment: pattern.fragment.as_deref().unwrap_or(self.fragment),
    })
  }

  pub(super) fn should_visit_dir(&self, path: &str) -> bool {
    let normalized = (!self.case_sensitive).then(|| normalize_case_insensitive_path(path));
    self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .any(|pattern| {
        if let Some(root) = &pattern.match_root {
          if root.starts_with(path) {
            return true;
          }
          if !Utf8Path::new(path).starts_with(root) {
            return false;
          }
        }
        pattern
          .case_insensitive_base
          .as_deref()
          .zip(normalized.as_deref())
          .is_none_or(|(base, path)| {
            is_same_or_descendant(base, path) || is_same_or_descendant(path, base)
          })
      })
  }

  pub(super) fn should_visit_skipped_dir(&self, path: &str) -> bool {
    let normalized = (!self.case_sensitive).then(|| normalize_case_insensitive_path(path));
    self
      .patterns
      .iter()
      .filter(|pattern| !pattern.negative)
      .any(|pattern| {
        pattern
          .match_root
          .as_ref()
          .is_some_and(|root| root.starts_with(path))
          || pattern
            .case_insensitive_base
            .as_deref()
            .zip(normalized.as_deref())
            .is_some_and(|(base, path)| is_same_or_descendant(base, path))
      })
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
