use std::{hash::BuildHasherDefault, sync::Arc};

use dashmap::DashMap;
use rspack_fs::ReadableFileSystem;
use rustc_hash::FxHasher;

use super::resolver_impl::Resolver;
use crate::{DependencyCategory, Resolve};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
// Actually this should be ResolveOptionsWithDependencyCategory, it's a mistake from webpack, but keep the alignment for easily find the code in webpack
pub struct ResolveOptionsWithDependencyType {
  /// Shared with the owner of the options. Resolvers are cached by the *content*
  /// of these options, so keeping a reference lets the hot lookup path reuse the
  /// options that the caller already holds instead of deep-copying the whole
  /// `Resolve` for every resolution.
  pub resolve_options: Option<Arc<Resolve>>,
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
  /// Whether resolved resources carry the raw package.json description.
  /// Builds whose rules cannot observe it skip materializing the mirror.
  description_json: bool,
}

impl ResolverFactory {
  pub fn clear_cache(&self) {
    self.resolver.clear_cache();
  }

  pub fn new(options: Resolve, fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self::new_with_description_json(options, fs, true)
  }

  /// `description_json` controls whether resolved resources carry a raw
  /// `package.json` JSON copy; see `ResolverFactory::description_json`.
  pub fn new_with_description_json(
    options: Resolve,
    fs: Arc<dyn ReadableFileSystem>,
    description_json: bool,
  ) -> Self {
    Self {
      base_options: options.clone(),
      resolver: Resolver::new_with_description_json(options, fs, description_json),
      resolvers: Default::default(),
      description_json,
    }
  }

  pub fn get(&self, options: ResolveOptionsWithDependencyType) -> Arc<Resolver> {
    if let Some(r) = self.resolvers.get(&options) {
      r.clone()
    } else {
      let base_options = self.base_options.clone();
      let merged_options = match &options.resolve_options {
        Some(o) => base_options.merge((**o).clone()),
        None => base_options,
      };
      let resolver = Arc::new(self.resolver.clone_with_options(merged_options, &options));
      self.resolvers.insert(options, resolver.clone());
      resolver
    }
  }

  /// Whether resolved resources carry the raw `package.json` description.
  pub fn description_json(&self) -> bool {
    self.description_json
  }
}
