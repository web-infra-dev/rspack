use std::{
  hash::BuildHasherDefault,
  sync::{Arc, OnceLock},
};

use dashmap::DashMap;
use rspack_fs::ReadableFileSystem;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHasher;

use super::resolver_impl::Resolver;
use crate::{DependencyCategory, Resolve, ResolverCache};

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
  /// Set once by the compiler when its cache is portable.
  relocatable_roots: OnceLock<Vec<Utf8PathBuf>>,
}

impl ResolverFactory {
  pub fn clear_cache(&self) {
    self.resolver.clear_cache();
  }

  pub fn set_resolver_cache(&self, cache: Option<ResolverCache>) {
    self.resolver.set_resolver_cache(cache);
  }

  pub fn new(options: Resolve, fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      base_options: options.clone(),
      resolver: Resolver::new(options, fs),
      resolvers: Default::default(),
      relocatable_roots: OnceLock::new(),
    }
  }

  pub fn set_relocatable_roots(&self, roots: Vec<Utf8PathBuf>) {
    let _ = self.relocatable_roots.set(roots);
    // A plugin can request a resolver before the compiler is created.
    self.resolvers.clear();
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
      let roots = self.relocatable_roots.get().cloned().unwrap_or_default();
      let resolver = Arc::new(
        self
          .resolver
          .clone_with_options(merged_options, &options, roots),
      );
      self.resolvers.insert(options, resolver.clone());
      resolver
    }
  }
}
