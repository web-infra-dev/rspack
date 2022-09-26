use crate::Resolve;
use nodejs_resolver::{ResolverCache, ResolverError};
use rspack_error::{Error, Result};
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

pub struct ResolverFactory {
  cache: Arc<ResolverCache>,
}

impl Default for ResolverFactory {
  fn default() -> Self {
    Self::new()
  }
}

impl ResolverFactory {
  pub fn new() -> Self {
    Self {
      cache: Arc::new(ResolverCache::default()),
    }
  }

  pub fn get(&self, options: Resolve) -> Resolver {
    Resolver(nodejs_resolver::Resolver::new(
      nodejs_resolver::ResolverOptions {
        extensions: options.extensions,
        alias: options.alias,
        prefer_relative: options.prefer_relative,
        symlinks: options.symlinks,
        main_files: options.main_files,
        main_fields: options.main_fields,
        browser_field: options.browser_field,
        condition_names: options.condition_names,
        enforce_extension: None,
        external_cache: Some(self.cache.clone()),
        description_file: Some(String::from("package.json")),
        tsconfig: None,
      },
    ))
  }
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
    // if cost > std::time::Duration::from_millis(1) {
    println!(
      "resolve cost: {:?}, path: {:?}, request: {:?}",
      start.elapsed(),
      path,
      request
    );
    // }

    result
  }
}
