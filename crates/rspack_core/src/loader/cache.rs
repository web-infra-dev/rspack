use std::sync::{Arc, RwLock};

use rspack_collections::Identifier;
use rspack_error::{Result, error};
use rspack_loader_runner::{AdditionalData, Content, LoaderContext};
use rspack_paths::{ArcPath, ArcPathMap, ArcPathSet, Utf8Path};
use rspack_sources::SourceMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::RunnerContext;

#[derive(Debug, Clone)]
pub struct LoaderCacheEntry {
  pub content: Content,
  pub source_map: Option<String>,
  pub additional_data: Option<AdditionalData>,
  pub file_dependencies: HashSet<std::path::PathBuf>,
  pub context_dependencies: HashSet<std::path::PathBuf>,
  pub missing_dependencies: HashSet<std::path::PathBuf>,
  pub build_dependencies: HashSet<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LoaderCacheKey {
  module: Identifier,
  loader_index: i32,
}

#[derive(Debug, Default)]
pub struct LoaderCache {
  entries: RwLock<ArcPathMap<HashMap<LoaderCacheKey, Arc<LoaderCacheEntry>>>>,
}

impl LoaderCache {
  fn get(
    &self,
    resource: &ArcPath,
    module: Identifier,
    loader_index: i32,
  ) -> Option<Arc<LoaderCacheEntry>> {
    self
      .entries
      .read()
      .expect("loader cache should not be poisoned")
      .get(resource)
      .and_then(|entries| {
        entries
          .get(&LoaderCacheKey {
            module,
            loader_index,
          })
          .cloned()
      })
  }

  fn insert(
    &self,
    resource: ArcPath,
    module: Identifier,
    loader_index: i32,
    entry: LoaderCacheEntry,
  ) {
    self
      .entries
      .write()
      .expect("loader cache should not be poisoned")
      .entry(resource)
      .or_default()
      .insert(
        LoaderCacheKey {
          module,
          loader_index,
        },
        Arc::new(entry),
      );
  }

  pub fn remove(&self, removed_files: &ArcPathSet) {
    let mut entries = self
      .entries
      .write()
      .expect("loader cache should not be poisoned");
    for file in removed_files {
      entries.remove(file);
    }
  }
}

#[derive(Debug, Clone)]
pub struct LoaderCacheContext {
  cache: Arc<LoaderCache>,
  modified_files: Arc<ArcPathSet>,
  removed_files: Arc<ArcPathSet>,
  module: Identifier,
  resource: Option<ArcPath>,
}

impl LoaderCacheContext {
  pub fn new(
    cache: Arc<LoaderCache>,
    modified_files: Arc<ArcPathSet>,
    removed_files: Arc<ArcPathSet>,
    module: Identifier,
    resource: Option<&Utf8Path>,
  ) -> Self {
    Self {
      cache,
      modified_files,
      removed_files,
      module,
      resource: resource.map(Into::into),
    }
  }

  pub fn get(&self, loader_index: i32) -> Option<Arc<LoaderCacheEntry>> {
    let resource = self.resource.as_ref()?;
    if self.modified_files.contains(resource) || self.removed_files.contains(resource) {
      return None;
    }
    self.cache.get(resource, self.module, loader_index)
  }

  pub fn insert(&self, loader_index: i32, entry: LoaderCacheEntry) {
    let Some(resource) = self.resource.clone() else {
      return;
    };
    self
      .cache
      .insert(resource, self.module, loader_index, entry);
  }

  pub fn restore(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<bool> {
    let Some(entry) = self.get(loader_context.loader_index) else {
      return Ok(false);
    };
    let source_map = entry
      .source_map
      .as_ref()
      .map(|source_map| SourceMap::from_json(source_map.clone()))
      .transpose()
      .map_err(|error| error!("Failed to restore cached loader source map: {error}"))?;
    loader_context.__finish_with((
      Some(entry.content.clone()),
      source_map,
      entry.additional_data.clone(),
    ));
    loader_context
      .file_dependencies
      .extend(entry.file_dependencies.iter().cloned());
    loader_context
      .context_dependencies
      .extend(entry.context_dependencies.iter().cloned());
    loader_context
      .missing_dependencies
      .extend(entry.missing_dependencies.iter().cloned());
    loader_context
      .build_dependencies
      .extend(entry.build_dependencies.iter().cloned());
    Ok(true)
  }

  pub fn store(&self, loader_context: &LoaderContext<RunnerContext>) {
    let Some(content) = loader_context.content().cloned() else {
      return;
    };
    self.insert(
      loader_context.loader_index,
      LoaderCacheEntry {
        content,
        source_map: loader_context.source_map().map(SourceMap::to_json),
        additional_data: loader_context.additional_data().cloned(),
        file_dependencies: loader_context.file_dependencies.clone(),
        context_dependencies: loader_context.context_dependencies.clone(),
        missing_dependencies: loader_context.missing_dependencies.clone(),
        build_dependencies: loader_context.build_dependencies.clone(),
      },
    );
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_collections::Identifier;
  use rspack_loader_runner::Content;
  use rspack_paths::{ArcPath, ArcPathSet};

  use super::{LoaderCache, LoaderCacheContext, LoaderCacheEntry};

  fn entry(content: &str) -> LoaderCacheEntry {
    LoaderCacheEntry {
      content: Content::String(content.to_string()),
      source_map: None,
      additional_data: None,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
    }
  }

  fn context(
    cache: Arc<LoaderCache>,
    modified_files: Arc<ArcPathSet>,
    removed_files: Arc<ArcPathSet>,
  ) -> LoaderCacheContext {
    LoaderCacheContext {
      cache,
      modified_files,
      removed_files,
      module: Identifier::from("module"),
      resource: Some(ArcPath::from("/project/src/index.js")),
    }
  }

  #[test]
  fn misses_modified_resources() {
    let cache = Arc::new(LoaderCache::default());
    let initial = context(cache.clone(), Default::default(), Default::default());
    initial.insert(0, entry("cached"));
    assert!(initial.get(0).is_some());

    let mut modified_files = ArcPathSet::default();
    modified_files.insert(ArcPath::from("/project/src/index.js"));
    let modified = context(cache, Arc::new(modified_files), Default::default());
    assert!(modified.get(0).is_none());
  }

  #[test]
  fn removes_entries_for_deleted_resources() {
    let cache = Arc::new(LoaderCache::default());
    let context = context(cache.clone(), Default::default(), Default::default());
    context.insert(0, entry("cached"));

    let mut removed_files = ArcPathSet::default();
    removed_files.insert(ArcPath::from("/project/src/index.js"));
    cache.remove(&removed_files);

    assert!(context.get(0).is_none());
  }
}
