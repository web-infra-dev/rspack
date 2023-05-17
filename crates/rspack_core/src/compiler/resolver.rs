use std::{
  hash::BuildHasherDefault,
  path::{Path, PathBuf},
  sync::Arc,
};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::DependencyType;
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
      base_options
        .clone()
        .to_inner_options(cache.clone(), false, DependencyCategory::Unknown),
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
        Some(o) => base_options.merge(o.clone()),
        None => base_options,
      };
      let normalized = merged_options.to_inner_options(
        self.cache.clone(),
        options.resolve_to_context,
        options.dependency_category,
      );
      let resolver = Arc::new(Resolver(nodejs_resolver::Resolver::new(normalized)));
      self.resolvers.insert(options, resolver.clone());
      resolver
    }
  }
}

#[derive(Debug)]
pub struct Resolver(pub(crate) nodejs_resolver::Resolver);

impl Resolver {
  pub fn resolve(&self, path: &Path, request: &str) -> nodejs_resolver::RResult<ResolveResult> {
    self.0.resolve(path, request)
  }

  pub fn options(&self) -> &nodejs_resolver::Options {
    &self.0.options
  }

  pub fn dependencies(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
    // There are some issues with this method
    // self.0.get_dependency_from_entry()
    (vec![], vec![])
  }
}
