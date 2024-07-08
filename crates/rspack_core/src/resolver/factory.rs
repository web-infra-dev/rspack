use std::{hash::BuildHasherDefault, sync::Arc};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use super::resolver_impl::Resolver;
use crate::{DependencyCategory, Resolve};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
// Actually this should be ResolveOptionsWithDependencyCategory, it's a mistake from webpack, but keep the alignment for easily find the code in webpack
pub struct ResolveOptionsWithDependencyType {
  pub resolve_options: Option<Box<Resolve>>,
  pub resolve_to_context: bool,
  pub dependency_category: DependencyCategory,
}

#[derive(Debug)]
pub struct ResolverFactory {
  base_options: Resolve,
  resolver: Resolver,
  /// Different resolvers are used for different resolution strategies such as ESM and CJS.
  /// All resolvers share the same underlying cache.
  resolvers: DashMap<ResolveOptionsWithDependencyType, Arc<Resolver>, BuildHasherDefault<FxHasher>>,
}

impl Default for ResolverFactory {
  fn default() -> Self {
    Self::new(Resolve::default())
  }
}

impl ResolverFactory {
  pub fn clear_cache(&self) {
    self.resolver.clear_cache();
  }

  pub fn new(options: Resolve) -> Self {
    Self {
      base_options: options.clone(),
      resolver: Resolver::new(options),
      resolvers: Default::default(),
    }
  }

  pub fn get(&self, options: ResolveOptionsWithDependencyType) -> Arc<Resolver> {
    if let Some(r) = self.resolvers.get(&options) {
      r.clone()
    } else {
      let base_options = self.base_options.clone();
      let merged_options = match &options.resolve_options {
        Some(o) => base_options.merge(*o.clone()),
        None => base_options,
      };
      let resolver = Arc::new(self.resolver.clone_with_options(merged_options, &options));
      self.resolvers.insert(options, resolver.clone());
      resolver
    }
  }
}
