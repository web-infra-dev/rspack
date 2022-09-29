use crate::Resolve;
use nodejs_resolver::{ResolverCache, ResolverError};
use rspack_error::{Error, Result};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub enum ResolveResult {
  Info(ResolveInfo),
  Ignored,
}

#[derive(Debug)]
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
  cache: Arc<ResolverCache>,
  base_options: Resolve,
}

impl Default for ResolverFactory {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl ResolverFactory {
  pub fn new(base_options: Resolve) -> Self {
    Self {
      cache: Arc::new(ResolverCache::default()),
      base_options,
    }
  }

  pub fn get(&self, options: Resolve) -> Resolver {
    let options = merge_resolver_options(&self.base_options, options);
    Resolver(nodejs_resolver::Resolver::new(
      nodejs_resolver::ResolverOptions {
        extensions: options.extensions,
        alias: options.alias,
        prefer_relative: options.prefer_relative,
        symlinks: options.symlinks,
        main_files: options.main_files,
        main_fields: options.main_fields,
        browser_field: options.browser_field,
        condition_names: HashSet::from_iter(options.condition_names),
        enforce_extension: None,
        external_cache: Some(self.cache.clone()),
        description_file: Some(String::from("package.json")),
        tsconfig: None,
      },
    ))
  }
}

fn merge_resolver_options(a: &Resolve, b: Resolve) -> Resolve {
  Resolve {
    alias: b.alias,
    prefer_relative: b.prefer_relative,
    symlinks: b.symlinks,
    browser_field: b.browser_field,
    extensions: merge_string_array(&a.extensions, b.extensions),
    main_files: merge_string_array(&a.main_files, b.main_files),
    main_fields: merge_string_array(&a.main_fields, b.main_fields),
    condition_names: merge_string_array(&a.condition_names, b.condition_names),
  }
}

fn merge_string_array(a: &[String], b: Vec<String>) -> Vec<String> {
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
    extensions: to_string(vec!["a", "b"]),
    alias: vec![("c".to_string(), AliasMap::Ignored)],
    prefer_relative: false,
    symlinks: true,
    main_files: to_string(vec!["d", "e", "f"]),
    main_fields: to_string(vec!["g", "h", "i"]),
    browser_field: true,
    condition_names: to_string(vec!["j", "k"]),
  };
  let another = Resolve {
    extensions: to_string(vec!["a1", "b1"]),
    alias: vec![("c2".to_string(), AliasMap::Ignored)],
    prefer_relative: true,
    symlinks: false,
    browser_field: true,
    main_files: to_string(vec!["d1", "e", "..."]),
    main_fields: to_string(vec!["...", "h"]),
    condition_names: to_string(vec!["..."]),
  };
  let options = merge_resolver_options(&base, another);
  assert_eq!(options.extensions, to_string(vec!["a1", "b1"]));
  assert!(options.prefer_relative);
  assert!(!options.symlinks);
  assert_eq!(options.main_files, vec!["d1", "e", "d", "e", "f"]);
  assert_eq!(options.main_fields, vec!["g", "h", "i", "h"]);
  assert_eq!(options.condition_names, vec!["j", "k"]);
}

#[test]
fn test_merge_string_array() {
  let base = to_string(vec!["base0", "base1"]);
  assert!(merge_string_array(&base, vec![]).is_empty());
  assert_eq!(
    merge_string_array(&base, to_string(vec!["a", "b"])),
    to_string(vec!["a", "b"])
  );
  assert_eq!(
    merge_string_array(&base, to_string(vec!["...", "a", "...", "b", "..."])),
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
  pub fn resolve(&self, path: &Path, request: &str) -> Result<ResolveResult> {
    let start = std::time::SystemTime::now();

    let result = self
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
      .map_err(|error| match error {
        ResolverError::Io(error) => Error::Io { source: error },
        ResolverError::UnexpectedJson((json_path, error)) => Error::Anyhow {
          source: anyhow::Error::msg(format!("{:?} in {:?}", error, json_path)),
        },
        ResolverError::UnexpectedValue(error) => Error::Anyhow {
          source: anyhow::Error::msg(error),
        },
        ResolverError::ResolveFailedTag => Error::BatchErrors(vec![]), // just for tag
      });

    let cost = start.elapsed().unwrap();
    if cost > std::time::Duration::from_micros(600) {
      println!(
        "resolve cost: {:?}, path: {:?}, request: {:?}",
        cost, path, request
      );
    }
    result
  }
}
