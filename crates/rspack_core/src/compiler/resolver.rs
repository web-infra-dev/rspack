use std::{
  hash::BuildHasherDefault,
  path::{Path, PathBuf},
  sync::Arc,
};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::{AliasMap, DependencyType};
use crate::{DependencyCategory, Resolve};

pub type ResolveResult = nodejs_resolver::ResolveResult<nodejs_resolver::Resource>;

#[derive(Debug)]
pub struct ResolverFactory {
  cache: Arc<nodejs_resolver::Cache>,
  base_options: Resolve,
  pub resolver: Resolver,
  resolvers: DashMap<ResolveOptionsWithDependencyType, Arc<Resolver>, BuildHasherDefault<FxHasher>>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResolveOptionsWithDependencyType {
  pub resolve_options: Option<Resolve>,
  pub resolve_to_context: bool,
  pub dependency_type: DependencyType,
  pub dependency_category: DependencyCategory,
}

impl Default for ResolverFactory {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl ResolverFactory {
  pub fn clear_entries(&self) {
    self.resolver.0.clear_entries();
  }

  pub fn new(base_options: Resolve) -> Self {
    let cache = Arc::new(nodejs_resolver::Cache::default());
    let resolver = Resolver(nodejs_resolver::Resolver::new(
      base_options.clone().to_inner_options(cache.clone(), false),
    ));
    Self {
      cache,
      base_options,
      resolvers: Default::default(),
      resolver,
    }
  }

  pub fn get(&self, options: ResolveOptionsWithDependencyType) -> Arc<Resolver> {
    if let Some(r) = self.resolvers.get(&options) {
      r.clone()
    } else {
      let base_options = self.base_options.clone();
      let merged_options = match &options.resolve_options {
        Some(o) => merge_resolver_options(base_options, o.clone()),
        None => match &self.base_options.condition_names {
          None => {
            let is_esm = matches!(options.dependency_category, DependencyCategory::Esm);
            let condition_names = if is_esm {
              vec![
                String::from("import"),
                String::from("module"),
                String::from("webpack"),
                String::from("development"),
                String::from("browser"),
              ]
            } else {
              vec![
                String::from("require"),
                String::from("module"),
                String::from("webpack"),
                String::from("development"),
                String::from("browser"),
              ]
            };
            let options = Resolve {
              condition_names: Some(condition_names),
              ..self.base_options.clone()
            };
            merge_resolver_options(base_options, options)
          }
          _ => base_options,
        },
      };
      let resolver = Arc::new(Resolver(nodejs_resolver::Resolver::new(
        merged_options.to_inner_options(self.cache.clone(), options.resolve_to_context),
      )));
      self.resolvers.insert(options, resolver.clone());
      resolver
    }
  }
}

fn merge_resolver_options(base: Resolve, other: Resolve) -> Resolve {
  fn overwrite<T, F>(a: Option<T>, b: Option<T>, f: F) -> Option<T>
  where
    T: Clone,
    F: FnOnce(T, T) -> T,
  {
    match (a, b) {
      (Some(a), Some(b)) => Some(f(a, b)),
      (Some(a), None) => Some(a),
      (None, Some(b)) => Some(b),
      (None, None) => None,
    }
  }

  let alias = overwrite(base.alias, other.alias, |pre, mut now| {
    now.extend(pre.into_iter());
    let now: indexmap::IndexSet<(String, Vec<AliasMap>)> = now.into_iter().collect();
    now.into_iter().collect()
  });
  let fallback = overwrite(base.fallback, other.fallback, |pre, mut now| {
    now.extend(pre.into_iter());
    let now: indexmap::IndexSet<(String, Vec<AliasMap>)> = now.into_iter().collect();
    now.into_iter().collect()
  });
  let prefer_relative = overwrite(base.prefer_relative, other.prefer_relative, |_, value| {
    value
  });
  let symlinks = overwrite(base.symlinks, other.symlinks, |_, value| value);
  let browser_field = overwrite(base.browser_field, other.browser_field, |_, value| value);
  let extensions = overwrite(base.extensions, other.extensions, |base, value| {
    normalize_string_array(&base, value)
  });
  let main_files = overwrite(base.main_files, other.main_files, |base, value| {
    normalize_string_array(&base, value)
  });
  let main_fields = overwrite(base.main_fields, other.main_fields, |base, value| {
    normalize_string_array(&base, value)
  });
  let modules = overwrite(base.modules, other.modules, |base, value| {
    normalize_string_array(&base, value)
  });
  let condition_names = overwrite(
    base.condition_names,
    other.condition_names,
    |base, value| normalize_string_array(&base, value),
  );
  let tsconfig = other.tsconfig;

  Resolve {
    fallback,
    modules,
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

#[cfg(test)]
mod test {
  use super::*;

  fn to_string(a: Vec<&str>) -> Vec<String> {
    a.into_iter().map(String::from).collect()
  }

  #[test]
  fn test_merge_resolver_options() {
    use crate::AliasMap;
    let base = Resolve {
      extensions: Some(to_string(vec!["a", "b"])),
      alias: Some(vec![("c".to_string(), vec![AliasMap::Ignored])]),
      symlinks: Some(false),
      main_files: Some(to_string(vec!["d", "e", "f"])),
      main_fields: Some(to_string(vec!["g", "h", "i"])),
      browser_field: Some(true),
      condition_names: Some(to_string(vec!["j", "k"])),
      ..Default::default()
    };
    let another = Resolve {
      extensions: Some(to_string(vec!["a1", "b1"])),
      alias: Some(vec![("c2".to_string(), vec![AliasMap::Ignored])]),
      prefer_relative: Some(true),
      browser_field: Some(true),
      main_files: Some(to_string(vec!["d1", "e", "..."])),
      main_fields: Some(to_string(vec!["...", "h", "..."])),
      condition_names: Some(to_string(vec!["f", "..."])),
      ..Default::default()
    };
    let options = merge_resolver_options(base, another);
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
    assert_eq!(
      options.alias.expect("should be Ok"),
      vec![
        ("c2".to_string(), vec![AliasMap::Ignored]),
        ("c".to_string(), vec![AliasMap::Ignored])
      ]
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
}

#[derive(Debug)]
pub struct Resolver(pub(crate) nodejs_resolver::Resolver);

impl Resolver {
  pub fn resolve(&self, path: &Path, request: &str) -> nodejs_resolver::RResult<ResolveResult> {
    self.0.resolve(path, request)
  }

  pub fn dependencies(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
    // There are some issues with this method
    // self.0.get_dependency_from_entry()
    (vec![], vec![])
  }
}
