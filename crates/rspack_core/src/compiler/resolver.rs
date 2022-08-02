use nodejs_resolver::ResolverCache;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashSet, path::PathBuf};

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

pub type AliasMap = nodejs_resolver::AliasMap;

#[derive(Debug)]
pub struct ResolverOptions {
  pub extensions: Vec<String>,
  pub enforce_extension: Option<bool>,
  pub alias: Vec<(String, AliasMap)>,
  pub prefer_relative: bool,
  pub symlinks: bool,
  pub description_file: Option<String>,
  pub main_files: Vec<String>,
  pub main_fields: Vec<String>,
  pub browser_field: bool,
  pub condition_names: HashSet<String>,
}

impl Default for ResolverOptions {
  fn default() -> Self {
    let inner = nodejs_resolver::ResolverOptions::default();
    Self {
      extensions: inner.extensions,
      enforce_extension: inner.enforce_extension,
      alias: inner.alias,
      prefer_relative: inner.prefer_relative,
      symlinks: inner.symlinks,
      description_file: inner.description_file,
      main_files: inner.main_files,
      main_fields: inner.main_fields,
      browser_field: true,
      condition_names: inner.condition_names,
    }
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

  pub fn get(&self, options: ResolverOptions) -> Resolver {
    Resolver(nodejs_resolver::Resolver::new(
      nodejs_resolver::ResolverOptions {
        extensions: options.extensions,
        enforce_extension: options.enforce_extension,
        alias: options.alias,
        prefer_relative: options.prefer_relative,
        external_cache: Some(self.cache.clone()),
        symlinks: options.symlinks,
        description_file: options.description_file,
        main_files: options.main_files,
        main_fields: options.main_fields,
        browser_field: options.browser_field,
        condition_names: options.condition_names,
        tsconfig: None,
      },
    ))
  }
}

#[derive(Debug)]
pub struct Resolver(pub(crate) nodejs_resolver::Resolver);

impl Resolver {
  pub fn resolve(&self, base_dir: &Path, request: &str) -> anyhow::Result<ResolveResult> {
    self
      .0
      .resolve(base_dir, request)
      .map(|inner_result| match inner_result {
        nodejs_resolver::ResolveResult::Info(info) => ResolveResult::Info(ResolveInfo {
          path: info.path,
          query: info.request.query.into(),
          fragment: info.request.fragment.into(),
        }),
        nodejs_resolver::ResolveResult::Ignored => ResolveResult::Ignored,
      })
      .map_err(anyhow::Error::msg)
  }
}
