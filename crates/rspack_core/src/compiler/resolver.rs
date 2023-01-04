use crate::Resolve;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, Clone)]
pub enum ResolveResult {
  Info(ResolveInfo),
  Ignored,
}

#[derive(Debug, Clone)]
pub struct ResolveInfo {
  pub path: PathBuf,
  pub query: String,
  pub fragment: String,
}

impl ResolveInfo {
  pub fn join(&self) -> String {
    format!("{}{}{}", self.path.display(), self.query, self.fragment)
  }
}

#[derive(Debug)]
pub struct ResolverFactory {
  cache: Arc<nodejs_resolver::Cache>,
  base_options: Resolve,
  resolvers: dashmap::DashMap<Resolve, Arc<Resolver>>,
}

impl Default for ResolverFactory {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl ResolverFactory {
  pub fn new(base_options: Resolve) -> Self {
    Self {
      cache: Arc::new(nodejs_resolver::Cache::default()),
      base_options,
      resolvers: Default::default(),
    }
  }

  pub fn get(&self, options: Resolve) -> Arc<Resolver> {
    if let Some(r) = self.resolvers.get(&options) {
      r.clone()
    } else {
      let base = self
        .base_options
        .clone()
        .to_inner_options(self.cache.clone());
      let merged_options = merge_resolver_options(base, options.clone());
      let resolver = Arc::new(Resolver(nodejs_resolver::Resolver::new(
        merged_options.to_inner_options(self.cache.clone()),
      )));
      self.resolvers.insert(options, resolver.clone());
      resolver
    }
  }
}

fn merge_resolver_options(base: nodejs_resolver::Options, other: Resolve) -> Resolve {
  fn overwrite<T: Clone, F: FnOnce(T, T) -> Option<T>>(a: T, b: Option<T>, f: F) -> Option<T> {
    match b {
      Some(value) => f(a, value),
      None => Some(a),
    }
  }

  let alias = overwrite(base.alias, other.alias, |_, value| Some(value));
  let prefer_relative = overwrite(base.prefer_relative, other.prefer_relative, |_, value| {
    Some(value)
  });
  let symlinks = overwrite(base.symlinks, other.symlinks, |_, value| Some(value));
  let browser_field = overwrite(base.browser_field, other.browser_field, |_, value| {
    Some(value)
  });
  let extensions = overwrite(base.extensions, other.extensions, |base, value| {
    Some(normalize_string_array(&base, value))
  });
  let main_files = overwrite(base.main_files, other.main_files, |base, value| {
    Some(normalize_string_array(&base, value))
  });
  let main_fields = overwrite(base.main_fields, other.main_fields, |base, value| {
    Some(normalize_string_array(&base, value))
  });
  let condition_names = overwrite(
    base.condition_names.into_iter().collect(),
    other.condition_names,
    |base, value| Some(normalize_string_array(&base, value)),
  );
  let tsconfig = other.tsconfig;
  Resolve {
    alias,
    prefer_relative,
    symlinks,
    browser_field,
    extensions,
    main_files,
    main_fields,
    condition_names,
    tsconfig,
  }
}

fn normalize_string_array(a: &[String], b: Vec<String>) -> Vec<String> {
  b.into_iter().fold(vec![], |mut acc, item| {
    if item.eq("...") {
      acc.append(&mut a.to_vec());
    } else {
      acc.push(item);
    }
    acc
  })
}

#[test]
fn test_merge_resolver_options() {
  use crate::AliasMap;
  let base = Resolve {
    extensions: Some(to_string(vec!["a", "b"])),
    alias: Some(vec![("c".to_string(), AliasMap::Ignored)]),
    symlinks: Some(false),
    main_files: Some(to_string(vec!["d", "e", "f"])),
    main_fields: Some(to_string(vec!["g", "h", "i"])),
    browser_field: Some(true),
    condition_names: Some(to_string(vec!["j", "k"])),
    ..Default::default()
  };
  let another = Resolve {
    extensions: Some(to_string(vec!["a1", "b1"])),
    alias: Some(vec![("c2".to_string(), AliasMap::Ignored)]),
    prefer_relative: Some(true),
    browser_field: Some(true),
    main_files: Some(to_string(vec!["d1", "e", "..."])),
    main_fields: Some(to_string(vec!["...", "h", "..."])),
    condition_names: Some(to_string(vec!["f", "..."])),
    ..Default::default()
  };
  let options = merge_resolver_options(base.to_inner_options(Default::default()), another);
  assert_eq!(
    options.extensions.expect("should be Ok"),
    to_string(vec!["a1", "b1"])
  );
  assert!(options.prefer_relative.expect("should be Ok"));
  assert!(!options.symlinks.expect("should be Ok"));
  assert_eq!(
    options.main_files.expect("should be Ok"),
    vec!["d1", "e", "d", "e", "f"]
  );
  assert_eq!(
    options.main_fields.expect("should be Ok"),
    vec!["g", "h", "i", "h", "g", "h", "i"]
  );
  assert_eq!(options.condition_names.expect("should be Ok").len(), 3);
}

#[test]
fn test_normalize_string_array() {
  let base = to_string(vec!["base0", "base1"]);
  assert!(normalize_string_array(&base, vec![]).is_empty());
  assert_eq!(
    normalize_string_array(&base, to_string(vec!["a", "b"])),
    to_string(vec!["a", "b"])
  );
  assert_eq!(
    normalize_string_array(&base, to_string(vec!["...", "a", "...", "b", "..."])),
    to_string(vec![
      "base0", "base1", "a", "base0", "base1", "b", "base0", "base1"
    ])
  );
}

#[cfg(test)]
fn to_string(a: Vec<&str>) -> Vec<String> {
  a.into_iter().map(String::from).collect()
}

#[derive(Debug)]
pub struct Resolver(pub(crate) nodejs_resolver::Resolver);

impl Resolver {
  pub fn clear(&self) {
    self.0.clear_entries();
  }

  #[instrument(name = "nodejs_resolver", skip_all)]
  pub fn resolve(&self, path: &Path, request: &str) -> nodejs_resolver::RResult<ResolveResult> {
    self
      .0
      .resolve(path, request)
      .map(|inner_result| match inner_result {
        nodejs_resolver::ResolveResult::Info(info) => ResolveResult::Info(ResolveInfo {
          path: info.path,
          query: info.request.query.into(),
          fragment: info.request.fragment.into(),
        }),
        nodejs_resolver::ResolveResult::Ignored => ResolveResult::Ignored,
      })
  }
}
