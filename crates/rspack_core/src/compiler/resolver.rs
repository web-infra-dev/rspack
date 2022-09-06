use nodejs_resolver::ResolverCache;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::Resolve;

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
    Resolver {
      resolver: nodejs_resolver::Resolver::new(nodejs_resolver::ResolverOptions {
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
      }),
    }
  }
}

/**
 * support to override default options in rule.resolve
 */
pub struct ResolveWithOptions {
  pub prefer_relative: Option<bool>,
}
#[derive(Debug)]
pub struct Resolver {
  resolver: nodejs_resolver::Resolver,
}

impl Resolver {
  pub fn resolve(&self, base_dir: &Path, request: &str) -> anyhow::Result<ResolveResult> {
    self
      .resolver
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
  pub fn with_options(&self, options: ResolveWithOptions) -> Resolver {
    let merged_options = nodejs_resolver::ResolverOptions {
      prefer_relative: options
        .prefer_relative
        .unwrap_or(self.resolver.options.prefer_relative),
      ..self.resolver.options.clone()
    };
    let resolver = nodejs_resolver::Resolver::new(merged_options);
    Self { resolver }
  }
}
